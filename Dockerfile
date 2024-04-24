# Use the official Rust image as the base image
FROM rust:latest as builder

# env variables
ENV ROCKET_ENV=production
ENV DATABASE_URL=DATABASE_URL
ENV DATABASE_NAME=DATABASE_NAME

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy the rest of the source code to the container
COPY . .

# Install the application
RUN cargo install --path .

# Build the application
RUN cargo build --release

# Start a new stage and use a smaller base image
FROM debian:buster-slim

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy the compiled binary from the builder stage to the final image
COPY --from=builder /usr/src/myapp/target/release/flussonic-publishing-handler .

# Expose the port that your Rocket application listens on
EXPOSE 8000

# Command to run the application
CMD ["./flussonic-publishing-handler"]
