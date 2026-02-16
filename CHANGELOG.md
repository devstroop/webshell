# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-02-16

### Added
- Initial release of WebShell
- Extracted terminal functionality from webide project
- Rust backend with Axum and Socket.IO
- React frontend with xterm.js
- PTY support using portable-pty
- Session management with timeout cleanup
- Multiple terminal sessions support
- Docker and docker-compose support
- Configurable terminal settings
- Dark terminal theme

### Features
- Web-based terminal accessible via browser
- Real-time terminal I/O with Socket.IO
- Multiple concurrent terminal sessions
- Automatic idle session cleanup
- Terminal resize support
- Cross-platform shell support (bash/zsh on Unix, PowerShell on Windows)

### Security
- ⚠️ Warning: This is a basic version without authentication
- Recommended for use in isolated/containerized environments only
