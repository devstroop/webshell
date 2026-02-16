# WebShell Frontend

React + TypeScript frontend for WebShell.

## Quick Start

```bash
# Install dependencies
npm install

# Start dev server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Configuration

Environment variables (`.env` file):

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_API_URL` | http://localhost:3000 | Backend API URL |

## Development

```bash
# Lint code
npm run lint

# Format code
npm run format
```

## Project Structure

```
src/
├── main.tsx       # Entry point
├── App.tsx        # Main app component
├── Terminal.tsx   # xterm.js terminal component
├── store.ts       # Zustand state management
├── App.css        # Styles
└── index.css      # Global styles
```

## Technologies

- **React 18** - UI framework
- **TypeScript** - Type safety
- **xterm.js** - Terminal emulator
- **Socket.IO** - Real-time communication
- **Zustand** - State management
- **Vite** - Build tool
- **Tailwind CSS** - Utility-first CSS
