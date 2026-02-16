# WebShell

A minimal web-based terminal with OS-native authentication. Pure Rust backend, vanilla JS frontend.

## Features

- **OS Authentication** - Login with system credentials (macOS/Linux)
- **WebSocket Terminal** - Real-time PTY via native WebSocket
- **Minimal Frontend** - ~100 lines vanilla JS, no build step
- **Single Binary** - One Rust executable serves everything

## Quick Start

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/webshell
```

Open http://localhost:2222 and login with your OS username/password.

## Architecture

```
Browser (xterm.js)
       │ WebSocket
       ▼
   Rust (Axum)
       │
   ┌───┴───┐
   │  PTY  │ ← OS Auth (dscl/su)
   └───┬───┘
       ▼
     Shell
```

## Project Structure

```
webshell/
├── src/
│   ├── main.rs      # HTTP server, WebSocket, routes
│   ├── auth.rs      # OS authentication & sessions
│   ├── config.rs    # Environment configuration
│   ├── types.rs     # WebSocket message types
│   └── terminal/    # PTY management
├── static/
│   └── index.html   # Login + terminal UI
├── Cargo.toml
└── Dockerfile
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 2222 | Server port |
| `WORKSPACE_DIR` | ~ | Terminal working directory |
| `RUST_LOG` | info | Log level |
| `WEBSHELL_HOST` | (none) | Pre-configured host (hides field if set) |
| `WEBSHELL_USER` | (none) | Pre-configured username (hides field if set) |
| `WEBSHELL_PASSWORD` | (none) | Pre-configured password (auto-login if set with user) |

### Examples

```bash
# Full login form (host, user, password)
cargo run

# Host pre-set, ask for user + password
WEBSHELL_HOST=127.0.0.1 cargo run

# Only ask for password
WEBSHELL_HOST=127.0.0.1 WEBSHELL_USER=admin cargo run

# Auto-login (direct to terminal)
WEBSHELL_HOST=localhost WEBSHELL_USER=admin WEBSHELL_PASSWORD=secret cargo run
```

## WebSocket Protocol

### Client → Server
- `term.open` - Open terminal `{id, cols, rows}`
- `term.input` - Send input `{id, input}`
- `term.resize` - Resize `{id, cols, rows}`
- `term.close` - Close terminal `{id}`

### Server → Client
- `shell.output` - Output data `{id, output}`
- `shell.exit` - Process exited `{id, code}`

## Docker

```bash
docker compose up
```

## Security

- Authenticates against OS users via `dscl` (macOS) or `su` (Linux)
- Session tokens stored server-side with 24h expiry
- WebSocket connections require valid session cookie
- **Auto-logout on disconnect** - Session invalidated when terminal closes

⚠️ **Warning:** Exposes shell access. Use in trusted environments only.

## License

MIT
