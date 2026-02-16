# WebShell Backend

Rust backend for WebShell - web-based terminal service.

## Quick Start

```bash
# Copy environment config
cp .env.example .env

# Build and run
cargo run

# Or with auto-reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run
```

## Configuration

Environment variables (`.env` file):

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 3000 | HTTP server port |
| `WORKSPACE_DIR` | /workspace | Terminal working directory |
| `MAX_TERMINALS` | 10 | Max terminals per connection |
| `IDLE_TIMEOUT` | 3600 | Session timeout in seconds |
| `RUST_LOG` | info | Log level (trace, debug, info, warn, error) |

## Development

```bash
# Run tests
cargo test

# Check code
cargo check

# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Build for release
cargo build --release
```

## Project Structure

```
src/
├── main.rs          # Entry point & HTTP server
├── config.rs        # Configuration management
└── terminal/        # Terminal/PTY module
    ├── mod.rs       # Module exports
    ├── error.rs     # Error types
    ├── pty.rs       # PTY manager (portable-pty)
    ├── session.rs   # Session lifecycle management
    └── socketio.rs  # Socket.IO event handlers
```

## Dependencies

- **axum** - Web framework
- **portable-pty** - Cross-platform PTY
- **socketioxide** - Socket.IO server
- **tokio** - Async runtime
- **tower-http** - HTTP middleware (CORS, static files)
