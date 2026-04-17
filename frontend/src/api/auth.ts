import axios from 'axios'
import type { AuthState } from '../types'

export async function login(username: string, password: string): Promise<AuthState> {
  const { data } = await axios.post<AuthState>('/auth/login', { username, password }, { withCredentials: true })
  return data
}

export async function logout(): Promise<void> {
  await axios.post('/auth/logout', {}, { withCredentials: true })
}
