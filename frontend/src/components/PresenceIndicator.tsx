import { usePresence } from '../hooks/usePresence'

export function PresenceIndicator({ memberId }: { memberId: string }) {
  const viewers = usePresence(memberId)
  if (viewers.length === 0) return null

  return (
    <div className="flex items-center gap-1 text-sm text-gray-500">
      <span className="w-2 h-2 rounded-full bg-green-400 inline-block" />
      {viewers.length === 1 ? `${viewers[0]} is also viewing` : `${viewers.length} others viewing`}
    </div>
  )
}
