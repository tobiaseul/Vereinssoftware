import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getMember, updateMember, deleteMember } from '../api/members'
import { MemberForm } from '../components/MemberForm'
import { ConflictDialog } from '../components/ConflictDialog'
import { PresenceIndicator } from '../components/PresenceIndicator'
import type { Member } from '../types'
import { AxiosError } from 'axios'

export function MemberDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [draft, setDraft] = useState<Partial<Member> | null>(null)
  const [conflict, setConflict] = useState<{ serverMember: Member; myDraft: Partial<Member> } | null>(null)

  const { data: member, isLoading } = useQuery({
    queryKey: ['member', id],
    queryFn: () => getMember(id!),
  })

  useEffect(() => {
    if (member && !draft) setDraft(member)
  }, [member])

  const { mutate: save, isPending: isSaving } = useMutation({
    mutationFn: (data: Partial<Member>) => updateMember(id!, data as Member),
    onSuccess: (updated: Member) => { qc.setQueryData(['member', id], updated); setDraft(updated) },
    onError: async (err: unknown) => {
      if (err instanceof AxiosError && err.response?.status === 409) {
        const serverMember = await getMember(id!)
        setConflict({ serverMember, myDraft: draft! })
      }
    },
  })

  const { mutate: remove } = useMutation({
    mutationFn: () => deleteMember(id!),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['members'] }); navigate('/members') },
  })

  if (isLoading || !draft || !member) return <div className="p-6">Loading...</div>

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">{member.first_name} {member.last_name}</h1>
        <PresenceIndicator memberId={id!} />
      </div>

      <MemberForm value={draft} onChange={setDraft} />

      <div className="flex gap-3 mt-6">
        <button onClick={() => save(draft)} disabled={isSaving}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">
          {isSaving ? 'Saving...' : 'Save'}
        </button>
        <button onClick={() => navigate('/members')} className="px-4 py-2 border rounded hover:bg-gray-50">
          Cancel
        </button>
        <button onClick={() => { if (window.confirm('Mark as left?')) remove() }}
          className="ml-auto px-4 py-2 text-red-600 border border-red-300 rounded hover:bg-red-50">
          Mark as Left
        </button>
      </div>

      {conflict && (
        <ConflictDialog
          myDraft={conflict.myDraft}
          serverMember={conflict.serverMember}
          onResolve={(resolved) => { setConflict(null); setDraft(resolved); save(resolved) }}
          onDiscard={() => { setConflict(null); setDraft(conflict.serverMember) }}
        />
      )}
    </div>
  )
}
