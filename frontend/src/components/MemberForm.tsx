import { useQuery } from '@tanstack/react-query'
import { getFieldDefinitions } from '../api/fieldDefinitions'
import type { Member, MembershipType } from '../types'

interface Props {
  value: Partial<Member>
  onChange: (updated: Partial<Member>) => void
  disabled?: boolean
}

export function MemberForm({ value, onChange, disabled }: Props) {
  const { data: fields = [] } = useQuery({ queryKey: ['field-definitions'], queryFn: getFieldDefinitions })

  const set = (key: keyof Member, val: unknown) => onChange({ ...value, [key]: val })

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium mb-1">First Name *</label>
          <input required disabled={disabled} className="border rounded w-full px-3 py-2"
            value={value.first_name ?? ''} onChange={e => set('first_name', e.target.value)} />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Last Name *</label>
          <input required disabled={disabled} className="border rounded w-full px-3 py-2"
            value={value.last_name ?? ''} onChange={e => set('last_name', e.target.value)} />
        </div>
      </div>
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium mb-1">Email</label>
          <input type="email" disabled={disabled} className="border rounded w-full px-3 py-2"
            value={value.email ?? ''} onChange={e => set('email', e.target.value || null)} />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Phone</label>
          <input disabled={disabled} className="border rounded w-full px-3 py-2"
            value={value.phone ?? ''} onChange={e => set('phone', e.target.value || null)} />
        </div>
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Membership Type *</label>
        <select required disabled={disabled} className="border rounded px-3 py-2"
          value={value.membership_type ?? ''} onChange={e => set('membership_type', e.target.value as MembershipType)}>
          <option value="">Select...</option>
          <option value="Aktiv">Aktiv</option>
          <option value="Passiv">Passiv</option>
          <option value="Ehrenmitglied">Ehrenmitglied</option>
        </select>
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Notes</label>
        <textarea disabled={disabled} className="border rounded w-full px-3 py-2" rows={3}
          value={value.notes ?? ''} onChange={e => set('notes', e.target.value || null)} />
      </div>

      {fields.length > 0 && (
        <div className="border-t pt-4">
          <h3 className="font-medium mb-3">Custom Fields</h3>
          <div className="space-y-3">
            {fields.map(f => (
              <div key={f.id}>
                <label className="block text-sm font-medium mb-1">{f.name}{f.required && ' *'}</label>
                {f.field_type === 'boolean' ? (
                  <input type="checkbox" disabled={disabled}
                    checked={!!(value.custom_fields as Record<string, unknown>)?.[f.name]}
                    onChange={e => set('custom_fields', { ...(value.custom_fields as Record<string, unknown>), [f.name]: e.target.checked })} />
                ) : (
                  <input type={f.field_type === 'number' ? 'number' : f.field_type === 'date' ? 'date' : 'text'}
                    disabled={disabled} className="border rounded w-full px-3 py-2"
                    value={(value.custom_fields as Record<string, unknown>)?.[f.name] as string ?? ''}
                    onChange={e => set('custom_fields', { ...(value.custom_fields as Record<string, unknown>), [f.name]: e.target.value || null })} />
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
