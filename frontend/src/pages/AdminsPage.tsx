// src/pages/AdminsPage.tsx
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getAdmins, createAdmin, deleteAdmin } from '../api/admins'
import { useAuth } from '../hooks/useAuth'
import type { AdminRole } from '../types'

export function AdminsPage() {
  const { auth } = useAuth()
  const qc = useQueryClient()
  const [form, setForm] = useState({ username: '', password: '', role: 'Admin' as AdminRole })
  const { data: admins = [] } = useQuery({ queryKey: ['admins'], queryFn: getAdmins })
  const add = useMutation({
    mutationFn: () => createAdmin(form),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['admins'] }); setForm({ username: '', password: '', role: 'Admin' }) }
  })
  const remove = useMutation({
    mutationFn: deleteAdmin,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['admins'] })
  })

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-4">Admin Users</h1>
      <div className="grid grid-cols-2 gap-2 mb-4">
        <input className="border rounded px-3 py-2" placeholder="Username" value={form.username} onChange={e => setForm(f => ({ ...f, username: e.target.value }))} />
        <input type="password" className="border rounded px-3 py-2" placeholder="Password" value={form.password} onChange={e => setForm(f => ({ ...f, password: e.target.value }))} />
        <select className="border rounded px-3 py-2" value={form.role} onChange={e => setForm(f => ({ ...f, role: e.target.value as AdminRole }))}>
          <option value="Admin">Admin</option>
          <option value="SuperAdmin">SuperAdmin</option>
        </select>
        <button onClick={() => add.mutate()} disabled={!form.username || !form.password || add.isPending}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Add Admin</button>
      </div>
      <ul className="space-y-2">
        {admins.map(a => (
          <li key={a.id} className="flex justify-between items-center border rounded px-3 py-2">
            <div>
              <span className="font-medium">{a.username}</span>
              <span className="ml-2 text-sm text-gray-500">{a.role}</span>
              {a.id === auth?.admin_id && <span className="ml-2 text-xs text-blue-500">(you)</span>}
            </div>
            {a.id !== auth?.admin_id && (
              <button onClick={() => { if (confirm(`Remove ${a.username}?`)) remove.mutate(a.id) }}
                className="text-red-600 hover:text-red-800 text-sm">Remove</button>
            )}
          </li>
        ))}
      </ul>
    </div>
  )
}
