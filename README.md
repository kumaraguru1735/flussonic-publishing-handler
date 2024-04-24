# Note
**This Project is API project build on Rust [Rocket Framework] and Mongodb**

---

## clone then install.

```sh
cargo install .
```
Environmental variable required you can use .env file on project location

```sh
DATABASE_URL="mongodb://localhost:27017"
DATABASE_NAME="flussonic"
```

Run

```sh
cargo run
```


# Flussonic On Publish Handler

This is a simple script that will be executed when a stream is published on Flussonic Media Server. It will check the stream key and return a response to Flussonic Media Server.

# Configuration on Flussonic Media Server

```shell
stream aaa/aaa {
  input publish://;
  on_publish http://127.0.0.1:8000/check-stream;
}
```
Docs for Flussonic Media Server: [https://flussonic.com/doc/](https://flussonic.com/doc/)

## **API ROUTE**

✅: required

🕊: not required

| METHOD | PATH            | PAYLOADS                               | NOTE |
|:-------|:----------------|:---------------------------------------|:-----|
| POST   | `/check-stream` | `ip`:✅ <br/> `proto`:✅  <br/> `name`:✅ | -    |
| GET    | `/stream-log`   | 🕊                                     | -    |                                                                                                                                                                                                                                                                                                                                                                  |

---


## **API RESPONSE**


# POST `/check-stream`

```json
{
  "message": "Stream key accepted"
}
```

# GET `/stream-log`

```json
[
  {
    "s.no": 1,
    "datetime": "24/04/2024, 08:07:10 PM",
    "streamkey": "aaa/aaa",
    "msg": "Stream key expired"
  },
  {
    "s.no": 2,
    "datetime": "24/04/2024, 08:07:11 PM",
    "streamkey": "bbb/bbb",
    "msg": "Stream key accepted"
  },
  {
    "s.no": 3,
    "datetime": "24/04/2024, 08:07:12 PM",
    "streamkey": "ccc/ccc",
    "msg": "Stream key not found"
  }
]
```

