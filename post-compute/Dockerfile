FROM rust:1.88-alpine3.22 AS builder

# Install build dependencies with pinned versions
RUN apk add --no-cache musl-dev=1.2.5-r12 openssl-dev=3.5.6-r0 openssl-libs-static=3.5.6-r0

WORKDIR /app

# Copy manifest and source files
COPY . .

# Build the application
RUN cargo build --release --bin tee-worker-post-compute

FROM alpine:3.22

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/tee-worker-post-compute .

# Run the application
ENTRYPOINT ["/app/tee-worker-post-compute"]
