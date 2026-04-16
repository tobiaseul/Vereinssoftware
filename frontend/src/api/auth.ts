import axios from 'axios'
import { setAccessToken } from './client'
import type { AuthState } from '../types'

export async function login(username: string, password: string): Promise<AuthState> {
  const { data } = await axios.post<AuthState>('/auth/login', { username, password }, { withCredentials: true })
  setAccessToken(data.access_token)
  return data
}

export async function logout(): Promise<void> {
  await axios.post('/auth/logout', {}, { withCredentials: true })
  setAccessToken(null)
}
