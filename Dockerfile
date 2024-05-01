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

# Install the musl tools
RUN apt-get update && apt-get install -y musl-tools

# Install the musl target
RUN rustup target add x86_64-unknown-linux-musl

# Build the application with static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Start a new stage and use a smaller base image
FROM scratch

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy the compiled binary from the builder stage to the final image
COPY --from=builder /usr/src/myapp/target/x86_64-unknown-linux-musl/release/flussonic-publishing-handler .

# Expose the port that your Rocket application listens on
EXPOSE 8000

# Command to run the application
CMD ["./flussonic-publishing-handler"]


