import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createMember } from '../api/members'
import { MemberForm } from '../components/MemberForm'
import type { Member } from '../types'

export function MemberNewPage() {
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [draft, setDraft] = useState<Partial<Member>>({})

  const { mutate, isPending } = useMutation({
    mutationFn: () => createMember(draft as Member),
    onSuccess: (m: Member) => { qc.invalidateQueries({ queryKey: ['members'] }); navigate(`/members/${m.id}`) },
  })

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">New Member</h1>
      <MemberForm value={draft} onChange={setDraft} />
      <div className="flex gap-3 mt-6">
        <button onClick={() => mutate()} disabled={isPending}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">
          {isPending ? 'Creating...' : 'Create Member'}
        </button>
        <button onClick={() => navigate('/members')} className="px-4 py-2 border rounded hover:bg-gray-50">
          Cancel
        </button>
      </div>
    </div>
  )
}
