# WebShell

A lightweight, standalone web-based terminal built with Rust and React.

---

## ✨ Features

- **Web-Based Terminal** - Access a shell from anywhere via browser
- **Multiple Sessions** - Create and manage multiple terminal sessions
- **Real-time Communication** - Socket.IO for low-latency terminal I/O
- **PTY Support** - Full pseudo-terminal with proper signal handling
- **Lightweight** - Minimal footprint, only terminal functionality
- **Theme Support** - Light/Dark terminal themes
- **Configurable** - Adjustable font, colors, scrollback buffer

---

## 🚀 Quick Start

### Using Docker (Recommended)

```bash
cd webshell
docker compose up
```

Open in browser: http://localhost:3000

### Development Setup

```bash
# Terminal 1: Start Backend
cd backend
cp .env.example .env
cargo run

# Terminal 2: Start Frontend
cd frontend
npm install
npm run dev
```

Open in browser: http://localhost:5173

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│            Browser (xterm.js)           │
│  ┌─────────────────────────────────┐    │
│  │   Terminal Emulator UI          │    │
│  └──────────────┬──────────────────┘    │
└─────────────────┼──────────────────────┘
                  │ Socket.IO
┌─────────────────┼──────────────────────┐
│   Rust Backend (Axum + socketioxide)   │
│  ┌──────────────┴──────────────────┐   │
│  │   Terminal Session Manager      │   │
│  │   ┌─────────────────────────┐   │   │
│  │   │   PTY (portable-pty)    │   │   │
│  │   └───────────┬─────────────┘   │   │
│  └───────────────┼─────────────────┘   │
└──────────────────┼──────────────────────┘
                   │
            ┌──────┴──────┐
            │   Shell     │
            │ (bash/zsh)  │
            └─────────────┘
```

---

## 📁 Project Structure

```
webshell/
├── backend/                # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── config.rs      # Configuration
│   │   └── terminal/      # Terminal/PTY logic
│   │       ├── mod.rs
│   │       ├── pty.rs     # PTY manager
│   │       ├── session.rs # Session management
│   │       └── socketio.rs# Socket.IO handlers
│   └── Cargo.toml
├── frontend/              # React frontend
│   ├── src/
│   │   ├── App.tsx        # Main app
│   │   ├── Terminal.tsx   # xterm.js component
│   │   └── store.ts       # State management
│   └── package.json
├── docker-compose.yml
└── README.md
```

---

## 🔧 Configuration

### Backend Environment

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 3000 | HTTP server port |
| `WORKSPACE_DIR` | /workspace | Terminal working directory |
| `MAX_TERMINALS` | 10 | Max terminals per connection |
| `IDLE_TIMEOUT` | 3600 | Session timeout in seconds |
| `RUST_LOG` | info | Log level |

### Frontend Environment

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_API_URL` | http://localhost:3000 | Backend URL |

---

## 🔌 Socket.IO Events

### Client → Server
- `term.open` - Open new terminal
- `term.input` - Send input to terminal
- `term.resize` - Resize terminal dimensions
- `term.close` - Close terminal

### Server → Client
- `shell.output` - Terminal output data
- `shell.exit` - Terminal process exited

---

## 🐳 Docker

```bash
# Build and run
docker compose up --build

# Production deployment
docker compose -f docker-compose.prod.yml up -d
```

---

## 🔒 Security Considerations

⚠️ **Warning:** This gives shell access to the host system. Deploy with caution:

- Run in isolated containers
- Use proper authentication (not included in this basic version)
- Limit network access
- Monitor resource usage
- Implement rate limiting
- Consider using a restricted shell

---

## 📝 License

MIT License - Same as webide project

---

## 🙏 Credits

Extracted and simplified from the [webide](../webide) project.

- [xterm.js](https://xtermjs.org/) - Terminal emulator
- [portable-pty](https://github.com/wez/wezterm/tree/main/pty) - Cross-platform PTY
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [socketioxide](https://github.com/Totodore/socketioxide) - Socket.IO for Rust
