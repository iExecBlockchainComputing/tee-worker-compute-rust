FROM rust:1.98-alpine3.24 AS builder

WORKDIR /app

# Copy manifest and source files
COPY . .

# Build the application
RUN cargo build --release --bin tee-worker-pre-compute

FROM alpine:3.24

# Set working directory
WORKDIR /app

RUN apk --no-cache upgrade

# Copy the binary from builder stage
COPY --from=builder /app/target/release/tee-worker-pre-compute .

# Run the application
ENTRYPOINT ["/app/tee-worker-pre-compute"]
