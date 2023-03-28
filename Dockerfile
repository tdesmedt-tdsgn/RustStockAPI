# Use the official Rust image as the base image
FROM rust:1.57 as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy your project files into the container
COPY . .

# Build the application in release mode
RUN cargo build --release

# Use the official Debian slim image as the runtime image
FROM debian:buster-slim

# Install necessary packages for runtime
RUN apt-get update && \
    apt-get install -y ca-certificates tzdata && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/local/bin

# Copy the binary from the builder stage to the runtime stage
COPY --from=builder /usr/src/app/target/release/RustStockAPI .

# Expose the port your application uses
EXPOSE 8080

# Set the entrypoint for the container
ENTRYPOINT ["./RustStockAPI"]
