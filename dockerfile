# Build stage
FROM rust:1-bookworm AS builder

WORKDIR /app

# Cache dependencies: build a dummy project first so `cargo build` only
# re-downloads/re-compiles crates when Cargo.toml / Cargo.lock change
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates tzdata && rm -rf /var/lib/apt/lists/*

ENV TZ=America/New_York

COPY --from=builder /app/target/release/database-security-synchronizer .

# Exec form so the binary is PID 1 and receives SIGTERM from `docker stop`
CMD ["./database-security-synchronizer"]
