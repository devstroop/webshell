import { create } from 'zustand'

export interface TerminalSession {
  id: string
  name: string
}

export interface TerminalSettings {
  fontSize: number
  fontFamily: string
  cursorStyle: 'block' | 'underline' | 'bar'
  cursorBlink: boolean
  scrollback: number
}

interface TerminalState {
  sessions: TerminalSession[]
  activeSessionId: string | null
  settings: TerminalSettings

  // Actions
  createSession: () => string
  closeSession: (id: string) => void
  setActiveSession: (id: string) => void
  updateSettings: (settings: Partial<TerminalSettings>) => void
}

const defaultSettings: TerminalSettings = {
  fontSize: 14,
  fontFamily: "'JetBrains Mono', 'Fira Code', Consolas, monospace",
  cursorStyle: 'block',
  cursorBlink: true,
  scrollback: 10000,
}

export const useTerminalStore = create<TerminalState>((set, get) => ({
  sessions: [],
  activeSessionId: null,
  settings: defaultSettings,

  createSession: () => {
    const id = crypto.randomUUID()
    const session: TerminalSession = {
      id,
      name: `Terminal ${get().sessions.length + 1}`,
    }
    set((state) => ({
      sessions: [...state.sessions, session],
      activeSessionId: id,
    }))
    return id
  },

  closeSession: (id) => {
    set((state) => {
      const newSessions = state.sessions.filter((s) => s.id !== id)
      let newActiveId = state.activeSessionId
      if (state.activeSessionId === id) {
        newActiveId = newSessions[newSessions.length - 1]?.id || null
      }
      return { sessions: newSessions, activeSessionId: newActiveId }
    })
  },

  setActiveSession: (id) => set({ activeSessionId: id }),

  updateSettings: (settings) =>
    set((state) => ({
      settings: { ...state.settings, ...settings },
    })),
}))
