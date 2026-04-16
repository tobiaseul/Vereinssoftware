import type { Member } from '../types'

interface Props {
  myDraft: Partial<Member>
  serverMember: Member
  onResolve: (resolved: Partial<Member>) => void
  onDiscard: () => void
}

const FIELDS: (keyof Member)[] = ['first_name', 'last_name', 'email', 'phone', 'membership_type', 'notes']

export function ConflictDialog({ myDraft, serverMember, onResolve, onDiscard }: Props) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl p-6 max-w-2xl w-full mx-4">
        <h2 className="text-xl font-bold mb-2 text-red-600">Edit Conflict</h2>
        <p className="text-sm text-gray-600 mb-4">
          Another admin saved changes while you were editing. Review the differences and choose which version to keep.
        </p>
        <table className="w-full text-sm border-collapse mb-4">
          <thead>
            <tr className="bg-gray-100">
              <th className="px-3 py-2 border text-left">Field</th>
              <th className="px-3 py-2 border text-left">Your version</th>
              <th className="px-3 py-2 border text-left">Server version</th>
            </tr>
          </thead>
          <tbody>
            {FIELDS.filter(f => myDraft[f] !== serverMember[f]).map(f => (
              <tr key={f}>
                <td className="px-3 py-2 border font-medium">{f}</td>
                <td className="px-3 py-2 border bg-yellow-50">{String(myDraft[f] ?? '—')}</td>
                <td className="px-3 py-2 border bg-green-50">{String(serverMember[f] ?? '—')}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="flex gap-3">
          <button onClick={() => onResolve({ ...serverMember, ...myDraft, version: serverMember.version })}
            className="px-4 py-2 bg-yellow-500 text-white rounded hover:bg-yellow-600">
            Keep my changes
          </button>
          <button onClick={onDiscard}
            className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700">
            Use server version
          </button>
        </div>
      </div>
    </div>
  )
}
