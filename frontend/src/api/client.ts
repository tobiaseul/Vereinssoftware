import axios, { AxiosError } from 'axios'

export const client = axios.create({ withCredentials: true })

let accessToken: string | null = null
export const setAccessToken = (t: string | null) => { accessToken = t }
export const getAccessToken = () => accessToken

client.interceptors.request.use(config => {
  if (accessToken) config.headers.Authorization = `Bearer ${accessToken}`
  return config
})

let refreshPromise: Promise<string> | null = null

client.interceptors.response.use(
  r => r,
  async (error: AxiosError) => {
    const original = error.config!
    if (error.response?.status === 401 && !(original as any)._retry) {
      ;(original as any)._retry = true
      if (!refreshPromise) {
        refreshPromise = axios.post<{ access_token: string }>('/auth/refresh', {}, { withCredentials: true })
          .then(r => { accessToken = r.data.access_token; return accessToken as string })
          .finally(() => { refreshPromise = null })
      }
      try {
        const token = await refreshPromise
        original.headers!.Authorization = `Bearer ${token}`
        return client(original)
      } catch {
        accessToken = null
        window.location.href = '/login'
      }
    }
    return Promise.reject(error)
  }
)
