import type { Member } from '../types'

interface Props {
  myDraft: Partial<Member>
  serverMember: Member
  onResolve: (resolved: Partial<Member>) => void
  onDiscard: () => void
}

export function ConflictDialog(_props: Props) { return null }
