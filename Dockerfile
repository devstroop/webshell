# Multi-stage build for Rust backend
FROM rust:1.75-slim as backend-builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy backend source
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/src ./src

# Build release binary
RUN cargo build --release

# Build frontend
FROM node:20-alpine as frontend-builder

WORKDIR /app

# Copy frontend source
COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

# Final runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    bash \
    && rm -rf /var/lib/apt/lists/*

# Copy backend binary
COPY --from=backend-builder /app/target/release/webshell-backend /app/webshell-backend

# Copy frontend build
COPY --from=frontend-builder /app/dist /app/frontend/dist

# Create workspace directory
RUN mkdir -p /workspace

# Expose port
EXPOSE 3000

# Set environment
ENV PORT=3000
ENV WORKSPACE_DIR=/workspace
ENV RUST_LOG=info

# Run the application
CMD ["/app/webshell-backend"]
