import { useEffect } from 'react'
import { Terminal as TerminalIcon, Plus, Trash2 } from 'lucide-react'
import { useTerminalStore } from './store'
import { Terminal } from './Terminal'
import './App.css'

function App() {
  const { sessions, activeSessionId, createSession, closeSession, setActiveSession } =
    useTerminalStore()

  // Create initial session on mount
  useEffect(() => {
    if (sessions.length === 0) {
      createSession()
    }
  }, [])

  const handleNewTerminal = () => {
    createSession()
  }

  const handleCloseTerminal = (id: string) => {
    closeSession(id)
  }

  const activeSession = sessions.find((s) => s.id === activeSessionId)

  return (
    <div className="app">
      {/* Header */}
      <div className="header">
        <div className="header-left">
          <div className="logo">
            <TerminalIcon size={20} />
            <span>WebShell</span>
          </div>
        </div>
        <div className="header-right">
          <button className="btn btn-primary" onClick={handleNewTerminal}>
            <Plus size={16} />
            New Terminal
          </button>
        </div>
      </div>

      {/* Terminal Area */}
      <div className="terminal-container">
        {/* Tabs */}
        {sessions.length > 0 && (
          <div className="tabs">
            {sessions.map((session) => (
              <div
                key={session.id}
                className={`tab ${session.id === activeSessionId ? 'active' : ''}`}
                onClick={() => setActiveSession(session.id)}
              >
                <TerminalIcon size={14} />
                <span>{session.name}</span>
                {sessions.length > 1 && (
                  <button
                    className="tab-close"
                    onClick={(e) => {
                      e.stopPropagation()
                      handleCloseTerminal(session.id)
                    }}
                  >
                    <Trash2 size={12} />
                  </button>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Terminal */}
        <div className="terminal-wrapper">
          {activeSession ? (
            <Terminal key={activeSession.id} sessionId={activeSession.id} />
          ) : (
            <div className="empty-state">
              <TerminalIcon className="empty-state-icon" />
              <p className="empty-state-text">No terminal session</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default App
