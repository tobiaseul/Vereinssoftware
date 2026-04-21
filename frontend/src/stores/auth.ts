import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { login as apiLogin, logout as apiLogout } from '../api/auth'
import { setAccessToken, client } from '../api/client'
import type { AuthState, Role } from '../types'

export const useAuthStore = defineStore('auth', () => {
  const auth = ref<AuthState | null>(null)
  const isLoading = ref(true)
  const financeRoles = ref<Role[]>([])

  const hasFinanceRole = computed(() => financeRoles.value.length > 0)

  async function loadFinanceRoles(adminId: string) {
    try {
      const roles = await client.get<Role[]>(`/api/v1/finance/admins/${adminId}/roles`).then(r => r.data)
      financeRoles.value = roles
    } catch (err) {
      financeRoles.value = []
    }
  }

  async function login(username: string, password: string) {
    const result = await apiLogin(username, password)
    auth.value = result
    setAccessToken(result.access_token)
    await loadFinanceRoles(result.admin_id)
  }

  async function logout() {
    try {
      await apiLogout()
    } finally {
      auth.value = null
      financeRoles.value = []
      setAccessToken(null)
    }
  }

  async function silentRefresh() {
    try {
      const { data } = await client.post<AuthState>('/auth/refresh')
      auth.value = data
      setAccessToken(data.access_token)
      await loadFinanceRoles(data.admin_id)
    } catch {
      auth.value = null
      financeRoles.value = []
    } finally {
      isLoading.value = false
    }
  }

  return { auth, isLoading, financeRoles, hasFinanceRole, login, logout, silentRefresh }
})
