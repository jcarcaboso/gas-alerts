# Use a Rust base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the entire project directory into the container
COPY . .

# Create a directory to store persisted information
RUN mkdir /data
VOLUME ["/data"]

# Build the application
RUN cargo build --release

# Use a slim Linux base image
FROM debian:buster-slim

# Create a directory to store persisted information
RUN mkdir /data
VOLUME ["/data"]

# Copy the built binary from the builder image
COPY --from=builder /app/target/release/ethereum_gas_alert /usr/local/bin/

# Set environment variables (if needed)
# ENV ENV_VAR_NAME=ENV_VAR_VALUE

# Expose any necessary ports
# EXPOSE 8080

# Run the application when the container starts
CMD ["ethereum_gas_alert"]
