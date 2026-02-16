import { useEffect, useRef } from 'react'
import { Terminal as XTerm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { SearchAddon } from '@xterm/addon-search'
import { io, Socket } from 'socket.io-client'
import { useTerminalStore } from './store'
import '@xterm/xterm/css/xterm.css'

interface TerminalProps {
  sessionId: string
}

export function Terminal({ sessionId }: TerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const terminalRef = useRef<XTerm | null>(null)
  const socketRef = useRef<Socket | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)
  const { settings } = useTerminalStore()

  // Initialize terminal
  useEffect(() => {
    if (!containerRef.current) return

    // Create terminal instance
    const terminal = new XTerm({
      cursorBlink: settings.cursorBlink,
      cursorStyle: settings.cursorStyle,
      fontSize: settings.fontSize,
      fontFamily: settings.fontFamily,
      scrollback: settings.scrollback,
      theme: getTerminalTheme(),
      allowTransparency: true,
      convertEol: true,
    })

    // Load addons
    const fitAddon = new FitAddon()
    const webLinksAddon = new WebLinksAddon()
    const searchAddon = new SearchAddon()

    terminal.loadAddon(fitAddon)
    terminal.loadAddon(webLinksAddon)
    terminal.loadAddon(searchAddon)

    // Open terminal in container
    terminal.open(containerRef.current)

    // Delay fit to ensure container has dimensions
    requestAnimationFrame(() => {
      fitAddon.fit()
    })

    terminalRef.current = terminal
    fitAddonRef.current = fitAddon

    // Connect to backend via Socket.IO
    const socket = io(getSocketUrl(), {
      path: '/socket.io',
      transports: ['websocket', 'polling'],
    })

    socketRef.current = socket

    socket.on('connect', () => {
      console.log('Terminal connected:', sessionId)

      // Open terminal session
      socket.emit('term.open', {
        id: sessionId,
        cols: terminal.cols,
        rows: terminal.rows,
      })
    })

    // Listen for shell output
    socket.on('shell.output', (data: { id: string; output: string }) => {
      if (data.id === sessionId) {
        terminal.write(data.output)
      }
    })

    socket.on('disconnect', () => {
      terminal.write('\r\n\x1b[33m[Disconnected - attempting to reconnect...]\x1b[0m\r\n')
    })

    socket.on('connect_error', (error) => {
      console.error('Socket error:', error)
      terminal.write('\r\n\x1b[31m[Connection error]\x1b[0m\r\n')
    })

    // Handle terminal input
    terminal.onData((data) => {
      socket.emit('term.input', { id: sessionId, input: data })
    })

    // Handle resize
    const handleResize = () => {
      fitAddon.fit()
      socket.emit('term.resize', {
        id: sessionId,
        cols: terminal.cols,
        rows: terminal.rows,
      })
    }

    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(handleResize)
    })

    resizeObserver.observe(containerRef.current)

    // Focus terminal
    terminal.focus()

    // Cleanup
    return () => {
      resizeObserver.disconnect()
      socket.emit('term.close', { id: sessionId })
      socket.disconnect()
      terminal.dispose()
    }
  }, [sessionId])

  // Update settings when they change
  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.options.fontSize = settings.fontSize
      terminalRef.current.options.fontFamily = settings.fontFamily
      terminalRef.current.options.cursorBlink = settings.cursorBlink
      terminalRef.current.options.cursorStyle = settings.cursorStyle
      fitAddonRef.current?.fit()
    }
  }, [settings])

  return (
    <div
      ref={containerRef}
      className="h-full w-full"
      onClick={() => terminalRef.current?.focus()}
    />
  )
}

function getSocketUrl(): string {
  // Use the same host for Socket.IO
  return window.location.origin
}

function getTerminalTheme(): Record<string, string> {
  return {
    background: '#1a1b26',
    foreground: '#d4d4d4',
    cursor: '#aeafad',
    cursorAccent: '#000000',
    selectionBackground: '#264f78',
    selectionForeground: '#ffffff',
    black: '#000000',
    red: '#cd3131',
    green: '#0dbc79',
    yellow: '#e5e510',
    blue: '#2472c8',
    magenta: '#bc3fbc',
    cyan: '#11a8cd',
    white: '#e5e5e5',
    brightBlack: '#666666',
    brightRed: '#f14c4c',
    brightGreen: '#23d18b',
    brightYellow: '#f5f543',
    brightBlue: '#3b8eea',
    brightMagenta: '#d670d6',
    brightCyan: '#29b8db',
    brightWhite: '#ffffff',
  }
}
