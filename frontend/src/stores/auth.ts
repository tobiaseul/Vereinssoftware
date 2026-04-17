import { defineStore } from 'pinia'
import { ref } from 'vue'
import { login as apiLogin, logout as apiLogout } from '../api/auth'
import { setAccessToken, client } from '../api/client'
import type { AuthState } from '../types'

export const useAuthStore = defineStore('auth', () => {
  const auth = ref<AuthState | null>(null)
  const isLoading = ref(true)

  async function login(username: string, password: string) {
    const result = await apiLogin(username, password)
    auth.value = result
    setAccessToken(result.access_token)
  }

  async function logout() {
    try {
      await apiLogout()
    } finally {
      auth.value = null
      setAccessToken(null)
    }
  }

  async function silentRefresh() {
    try {
      const { data } = await client.post<AuthState>('/auth/refresh')
      auth.value = data
      setAccessToken(data.access_token)
    } catch {
      auth.value = null
    } finally {
      isLoading.value = false
    }
  }

  return { auth, isLoading, login, logout, silentRefresh }
})
