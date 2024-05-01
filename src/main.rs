// This is a simple Rocket application that checks if a stream key is valid or not.
// The stream key is stored in a MongoDB database. The stream key has the following fields: streamkey, username, suspend, streamType, blacklist, and end_duration.
#[macro_use] extern crate rocket;

use mongodb::{bson,Client, options::ClientOptions};
use rocket::{Request, response::status};
use rocket::http::Status;
use bson::{doc, Document};
use serde::{Deserialize, Serialize};
use chrono::{Utc, NaiveDateTime, TimeZone, Local};
use chrono_tz::Asia::Kolkata;
use rocket::serde::json::Json;
use std::env;
use dotenv::dotenv;
use std::sync::{Arc, Mutex};


#[derive(Debug, Serialize)]
struct StreamKeyResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
struct PublishData {
    ip: String,
    name: String,
}

#[derive(Debug, Serialize, Clone)]
struct StreamKeyLogMessage {
    s_no: usize,
    datetime: String,
    streamkey: String,
    msg: String,
}

lazy_static::lazy_static! {
    static ref STREAM_LOG_LIST: Arc<Mutex<Vec<StreamKeyLogMessage>>> = Arc::new(Mutex::new(Vec::new()));
    static ref STREAM_LOG_NO: Arc<Mutex<usize>> = Arc::new(Mutex::new(1));
}

fn streamkey_log(streamkey: String, msg: String) {
    let mut stream_log_list = STREAM_LOG_LIST.lock().unwrap();
    let mut stream_log_no = STREAM_LOG_NO.lock().unwrap();

    let datetime_now = Local::now().format("%d/%m/%Y, %I:%M:%S %p").to_string();

    let log_msg = StreamKeyLogMessage {
        s_no: *stream_log_no,
        datetime: datetime_now,
        streamkey,
        msg,
    };

    stream_log_list.push(log_msg);

    if stream_log_list.len() > 100 {
        stream_log_list.remove(0);
    }

    *stream_log_no += 1;
}



#[get("/stream-log")]
fn get_stream_log() -> Json<Vec<StreamKeyLogMessage>> {
    let stream_log_list = STREAM_LOG_LIST.lock().unwrap();
    Json(stream_log_list.clone())
}
async fn check_stream_key(publish_data: &PublishData) -> Result<bool, String> {
    println!("{:?}", publish_data);
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found in .env");
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_URL not found in .env");
    // Connect to MongoDB
    let client_options = ClientOptions::parse(database_url).await
        .map_err(|e| format!("Error connecting to MongoDB: {}", e))?;
    let client = Client::with_options(client_options)
        .map_err(|e| format!("Error creating MongoDB client: {}", e))?;

    // Access the collection
    let db = client.database(&*database_name);
    let collection = db.collection::<Document>("streamkeys");

    // Query to find the stream key
    let filter = doc! {
        "streamkey": publish_data.name.split('/').nth(1).unwrap(),
        "username": publish_data.name.split('/').nth(0).unwrap(),
        "suspend": false,
        "blacklist": { "$nin": [ &publish_data.ip ] }
    };

    let stream_key = collection.find_one(filter, None)
        .await
        .map_err(|e| format!("Error finding stream key: {}", e))?;

    println!("{:?}", stream_key);

    if let Some(stream_key) = stream_key {

        // Get the current time in Kolkata timezone
        let kolkata_now = Kolkata.from_utc_datetime(&Utc::now().naive_utc()).naive_utc();
        println!("Now {:?}", kolkata_now);

        // Check stream key expiration
        println!("Check stream key expiration");
        let end_duration_str = stream_key.get_str("end_duration").unwrap();
        println!("End Duration on db {:?}", end_duration_str);

        let parsed_end_duration = NaiveDateTime::parse_from_str(end_duration_str, "%d/%m/%Y, %I:%M:%S %p")
            .expect("Error parsing end duration");

        // Convert the end duration to Kolkata timezone
        let kolkata_end_duration = Kolkata
            .from_local_datetime(&parsed_end_duration)
            .earliest()
            .unwrap()
            .naive_utc();
        println!("Expired Duration {:?}", kolkata_end_duration);

        // Check if the stream key has expired
        if kolkata_end_duration <= kolkata_now {
            println!("Streamer Expired");
            streamkey_log(publish_data.name.clone(), String::from("Stream key expired"));
            return Ok(false); // Stream key expired
        }
        println!("Streamer valid");
        streamkey_log(publish_data.name.clone(), String::from("Stream key accepted"));
        return Ok(true); // Stream key valid
    }
    println!("Streamer not found");
    streamkey_log(publish_data.name.clone(), String::from("Stream key not found"));
    Ok(false) // Stream key not found
}

#[post("/check-stream", data = "<publish_data>")]
async fn check_data_handler(publish_data: Json<PublishData>) -> Result<status::Custom<Json<StreamKeyResponse>>, status::Custom<Json<StreamKeyResponse>>> {
    match check_stream_key(&publish_data.into_inner()).await {
        Ok(true) => Ok(status::Custom(Status::Ok, Json(StreamKeyResponse {
            message: String::from("Stream key accepted")
        }))),
        Ok(false) => Ok(status::Custom(Status::InternalServerError, Json(StreamKeyResponse {
            message: String::from("Stream key invalid or expired")
        }))),
        Err(e) => Err(status::Custom(Status::InternalServerError, Json(StreamKeyResponse {
            message: e
        }))),
    }
}


#[catch(500)]
fn internal_error() -> &'static str {
    "Internal server error"
}

#[catch(default)]
fn not_found(req: &Request) -> String {
    format!("{} not found", req.uri())
}

#[rocket::main]
async fn main() {
    dotenv().ok();
    rocket::build()
        .mount("/", routes![check_data_handler, get_stream_log])
        .register("/", catchers![internal_error, not_found])
        .launch()
        .await
        .expect("Failed to launch Rocket server");
}