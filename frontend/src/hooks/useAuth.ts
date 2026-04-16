import { createContext, useContext, useState, useEffect, createElement } from 'react'
import type { ReactNode } from 'react'
import { login as apiLogin, logout as apiLogout } from '../api/auth'
import { setAccessToken } from '../api/client'
import type { AuthState } from '../types'

interface AuthCtx {
  auth: AuthState | null
  login: (username: string, password: string) => Promise<void>
  logout: () => Promise<void>
  isLoading: boolean
}

const AuthContext = createContext<AuthCtx>(null!)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [auth, setAuth] = useState<AuthState | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    // Try silent refresh on load
    import('axios').then(({ default: axios }) =>
      axios.post<AuthState>('/auth/refresh', {}, { withCredentials: true })
        .then(r => { setAccessToken(r.data.access_token); setAuth(r.data) })
        .catch(() => {})
        .finally(() => setIsLoading(false))
    )
  }, [])

  const login = async (username: string, password: string) => {
    const data = await apiLogin(username, password)
    setAuth(data)
  }

  const logout = async () => {
    await apiLogout()
    setAuth(null)
  }

  return createElement(AuthContext.Provider, { value: { auth, login, logout, isLoading } }, children)
}

export const useAuth = () => useContext(AuthContext)
