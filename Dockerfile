# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates bash && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/webshell /app/webshell
COPY static /app/static

RUN mkdir -p /workspace

EXPOSE 2222

ENV PORT=2222
ENV WORKSPACE_DIR=/workspace
ENV WEBSHELL_STATIC_DIR=/app/static
ENV RUST_LOG=info

CMD ["/app/webshell"]
