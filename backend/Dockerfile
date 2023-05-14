# Builder stage
FROM rust:latest as builder

WORKDIR /var/www/app

# Copy the Cargo.toml and Cargo.lock files to cache dependencies
COPY ./Cargo.toml ./Cargo.lock ./

# Build a dummy application to pre-download and cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -fr src

COPY ./src ./src
RUN cargo build --release

# Runtime stage
FROM debian:buster-slim

WORKDIR /var/www/app

# Copy the built binary from the builder stage
COPY --from=builder /var/www/app/target/release .

# # Set the entry point for the container
CMD ["./rust-blockchain-example"]
