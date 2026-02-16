# Contributing to WebShell

Thank you for your interest in contributing to WebShell!

## Development Setup

1. **Prerequisites**
   - Rust 1.75+ (install via [rustup](https://rustup.rs/))
   - Node.js 20+
   - Docker (optional, for containerized development)

2. **Clone and Setup**
   ```bash
   cd webshell
   
   # Backend
   cd backend
   cp .env.example .env
   cargo build
   
   # Frontend
   cd ../frontend
   npm install
   ```

3. **Run in Development Mode**
   ```bash
   # Terminal 1: Backend
   cd backend
   cargo run
   
   # Terminal 2: Frontend
   cd frontend
   npm run dev
   ```

## Code Style

### Rust
- Follow standard Rust formatting: `cargo fmt`
- Pass Clippy lints: `cargo clippy`
- Add tests for new functionality

### TypeScript/React
- Use TypeScript strict mode
- Follow existing code patterns
- Use functional components with hooks

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Testing

```bash
# Backend tests
cd backend
cargo test

# Frontend tests (if added)
cd frontend
npm test
```

## Areas for Contribution

- Authentication & authorization
- Rate limiting
- Terminal settings UI
- Keyboard shortcuts
- Copy/paste improvements
- Search functionality
- File upload/download
- Terminal recording/playback
- Security hardening
- Performance optimizations
- Documentation improvements

## Questions?

Feel free to open an issue for discussion!
