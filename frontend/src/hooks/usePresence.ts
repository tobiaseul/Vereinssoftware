import { useEffect, useState, useRef } from 'react'
import { getAccessToken } from '../api/client'

export function usePresence(memberId: string) {
  const [viewers, setViewers] = useState<string[]>([])
  const wsRef = useRef<WebSocket | null>(null)

  useEffect(() => {
    const token = getAccessToken()
    if (!token) return

    const ws = new WebSocket(`ws://localhost:3000/ws?token=${token}`)
    wsRef.current = ws

    ws.onopen = () => {
      ws.send(JSON.stringify({ type: 'viewing', member_id: memberId }))
    }

    ws.onmessage = (e) => {
      try {
        const event = JSON.parse(e.data)
        if (event.type === 'viewing' && event.member_id === memberId) {
          setViewers(prev => [...new Set([...prev, event.username ?? 'Someone'])])
        }
        if (event.type === 'left' && event.member_id === memberId) {
          setViewers(prev => prev.filter(v => v !== event.username))
        }
      } catch {}
    }

    return () => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'left', member_id: memberId }))
      }
      ws.close()
    }
  }, [memberId])

  return viewers
}
