import { ref, onMounted, onUnmounted } from 'vue'
import { getAccessToken } from '../api/client'

export function usePresence(memberId: string) {
  const viewers = ref<string[]>([])
  let ws: WebSocket | null = null

  onMounted(() => {
    const token = getAccessToken()
    if (!token) return
    ws = new WebSocket(`ws://localhost:3000/ws?token=${token}`)

    ws.onopen = () => {
      ws!.send(JSON.stringify({ type: 'viewing', member_id: memberId }))
    }

    ws.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data)
        if (msg.viewers) viewers.value = msg.viewers
      } catch {
        // ignore malformed messages
      }
    }
  })

  onUnmounted(() => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'left' }))
      ws.close()
    }
  })

  return { viewers }
}
