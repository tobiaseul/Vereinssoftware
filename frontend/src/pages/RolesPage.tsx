// src/pages/RolesPage.tsx
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getRoles, createRole, deleteRole } from '../api/roles'

export function RolesPage() {
  const qc = useQueryClient()
  const [name, setName] = useState('')
  const { data: roles = [] } = useQuery({ queryKey: ['roles'], queryFn: getRoles })
  const add = useMutation({
    mutationFn: () => createRole(name),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['roles'] }); setName('') }
  })
  const remove = useMutation({
    mutationFn: deleteRole,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['roles'] })
  })

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-4">Roles</h1>
      <div className="flex gap-2 mb-4">
        <input className="border rounded px-3 py-2 flex-1" placeholder="New role name..." value={name} onChange={e => setName(e.target.value)} />
        <button onClick={() => add.mutate()} disabled={!name || add.isPending} className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Add</button>
      </div>
      <ul className="space-y-2">
        {roles.map(r => (
          <li key={r.id} className="flex justify-between items-center border rounded px-3 py-2">
            <span>{r.name}</span>
            <button onClick={() => remove.mutate(r.id)} className="text-red-600 hover:text-red-800 text-sm">Remove</button>
          </li>
        ))}
      </ul>
    </div>
  )
}
