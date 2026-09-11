# Build stage
FROM rust:1.75 AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (important for Rust + TLS)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/database-security-initializer .

# Run at container startup, NOT build time
CMD ["./database-security-initializer"]
