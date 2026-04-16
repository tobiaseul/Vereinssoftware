import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { getMembers, exportMembers } from '../api/members'

export function MembersPage() {
  const navigate = useNavigate()
  const [search, setSearch] = useState('')
  const [membershipType, setMembershipType] = useState('')
  const [includeLeft, setIncludeLeft] = useState(false)

  const { data: members = [], isLoading } = useQuery({
    queryKey: ['members', search, membershipType, includeLeft],
    queryFn: () => getMembers({ search: search || undefined, membership_type: membershipType || undefined, include_left: includeLeft }),
  })

  return (
    <div className="p-6 max-w-5xl mx-auto">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">Members ({members.length})</h1>
        <div className="flex gap-2">
          <button onClick={() => exportMembers({ search: search || undefined, membership_type: membershipType || undefined })}
            className="px-3 py-2 border rounded hover:bg-gray-50 text-sm">Export CSV</button>
          <button onClick={() => navigate('/members/new')}
            className="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm">+ New Member</button>
        </div>
      </div>

      <div className="flex gap-3 mb-4">
        <input placeholder="Search name..." value={search} onChange={e => setSearch(e.target.value)}
          className="border rounded px-3 py-2 flex-1" />
        <select value={membershipType} onChange={e => setMembershipType(e.target.value)} className="border rounded px-3 py-2">
          <option value="">All types</option>
          <option value="Aktiv">Aktiv</option>
          <option value="Passiv">Passiv</option>
          <option value="Ehrenmitglied">Ehrenmitglied</option>
        </select>
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={includeLeft} onChange={e => setIncludeLeft(e.target.checked)} />
          Include former
        </label>
      </div>

      {isLoading ? <p>Loading...</p> : (
        <table className="w-full border-collapse">
          <thead>
            <tr className="bg-gray-100 text-left text-sm">
              <th className="px-3 py-2 border">Name</th>
              <th className="px-3 py-2 border">Email</th>
              <th className="px-3 py-2 border">Type</th>
              <th className="px-3 py-2 border">Joined</th>
              <th className="px-3 py-2 border">Status</th>
            </tr>
          </thead>
          <tbody>
            {members.map(m => (
              <tr key={m.id} onClick={() => navigate(`/members/${m.id}`)}
                className="hover:bg-blue-50 cursor-pointer">
                <td className="px-3 py-2 border">{m.last_name}, {m.first_name}</td>
                <td className="px-3 py-2 border text-gray-600">{m.email ?? '—'}</td>
                <td className="px-3 py-2 border">{m.membership_type}</td>
                <td className="px-3 py-2 border">{m.joined_at}</td>
                <td className="px-3 py-2 border">{m.left_at ? <span className="text-red-500">Left</span> : <span className="text-green-600">Active</span>}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  )
}
