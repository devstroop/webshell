# WebShell

A minimal web-based terminal with OS-native authentication and SSH support. Pure Rust backend, vanilla JS frontend.

## Features

- **Local + Remote** - Connect to localhost (PTY) or remote servers (SSH)
- **Flexible Auth** - Password or SSH key authentication
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

Open http://localhost:2222 and login.

## Architecture

```
Browser (xterm.js)
       │ WebSocket
       ▼
   Rust (Axum)
       │
   ┌───┴───────────────┐
   │   Connection      │
   │   Router          │
   └───┬───────┬───────┘
       │       │
   ┌───┴───┐ ┌─┴────┐
   │ Local │ │ SSH  │
   │  PTY  │ │Client│
   └───────┘ └──────┘
```

## Project Structure

```
webshell/
├── src/
│   ├── main.rs      # HTTP server, WebSocket, routes
│   ├── auth.rs      # OS authentication & sessions
│   ├── config.rs    # Environment configuration
│   ├── ssh.rs       # SSH client for remote connections
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
| `WEBSHELL_HOST` | (none) | Target host (localhost = local PTY, else SSH) |
| `WEBSHELL_USER` | (none) | Username for connection |
| `WEBSHELL_PASSWORD` | (none) | Password authentication |
| `WEBSHELL_SSH_KEY` | (none) | Path to SSH private key file |
| `WEBSHELL_SSH_KEY_DATA` | (none) | SSH private key content (for secrets managers) |
| `WEBSHELL_SSH_PASSPHRASE` | (none) | Passphrase for encrypted SSH keys |

### Examples

```bash
# Full login form (host, user, password)
cargo run

# Local terminal with pre-set user
WEBSHELL_HOST=localhost WEBSHELL_USER=admin cargo run

# SSH with password (auto-login)
WEBSHELL_HOST=127.0.0.1 WEBSHELL_USER=admin WEBSHELL_PASSWORD=secret cargo run

# SSH with key file (auto-login)
WEBSHELL_HOST=127.0.0.1 WEBSHELL_USER=admin WEBSHELL_SSH_KEY=~/.ssh/id_rsa cargo run

# SSH with encrypted key
WEBSHELL_HOST=127.0.0.1 WEBSHELL_USER=admin \
  WEBSHELL_SSH_KEY=~/.ssh/id_rsa WEBSHELL_SSH_PASSPHRASE=keypass cargo run
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
