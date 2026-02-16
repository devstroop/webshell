# WebShell (Pure Rust)

A lightweight, standalone web-based terminal built entirely in Rust using Leptos and Axum.

## Architecture

```
webshell/
├── crates/
│   ├── shared/         # Shared types (messages, configs)
│   ├── backend/        # Axum server + PTY handling
│   └── frontend/       # Leptos UI (compiles to WASM)
├── Cargo.toml          # Workspace configuration
└── README.md
```

## Prerequisites

- Rust (latest stable)
- [Trunk](https://trunkrs.dev/) for building the frontend
- wasm32-unknown-unknown target

```bash
# Install trunk
cargo install trunk

# Add WASM target
rustup target add wasm32-unknown-unknown
```

## Development

### Backend

```bash
cd crates/backend
cargo run
```

The backend starts on http://localhost:3000

### Frontend

```bash
cd crates/frontend
trunk serve --open
```

The frontend dev server starts on http://localhost:8080 with proxy to backend.

## Production Build

```bash
# Build frontend
cd crates/frontend
trunk build --release

# Build backend
cd crates/backend
cargo build --release

# Run backend (serves frontend from dist/)
./target/release/webshell-backend
```

## Stack

- **Backend**: Axum + portable-pty (native WebSocket)
- **Frontend**: Leptos (CSR) + xterm.js
- **Communication**: Native WebSocket (not Socket.IO)
