// src/pages/FieldsPage.tsx
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getFieldDefinitions, createFieldDefinition, deleteFieldDefinition } from '../api/fieldDefinitions'
import type { FieldType } from '../types'

export function FieldsPage() {
  const qc = useQueryClient()
  const [form, setForm] = useState({ name: '', field_type: 'text' as FieldType, required: false })
  const { data: fields = [] } = useQuery({ queryKey: ['field-definitions'], queryFn: getFieldDefinitions })
  const add = useMutation({
    mutationFn: () => createFieldDefinition(form),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setForm({ name: '', field_type: 'text', required: false }) }
  })
  const remove = useMutation({
    mutationFn: deleteFieldDefinition,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['field-definitions'] })
  })

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-4">Custom Fields</h1>
      <div className="flex gap-2 mb-4">
        <input className="border rounded px-3 py-2 flex-1" placeholder="Field name..." value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} />
        <select className="border rounded px-3 py-2" value={form.field_type} onChange={e => setForm(f => ({ ...f, field_type: e.target.value as FieldType }))}>
          <option value="text">Text</option>
          <option value="number">Number</option>
          <option value="date">Date</option>
          <option value="boolean">Boolean</option>
        </select>
        <label className="flex items-center gap-1 text-sm">
          <input type="checkbox" checked={form.required} onChange={e => setForm(f => ({ ...f, required: e.target.checked }))} /> Required
        </label>
        <button onClick={() => add.mutate()} disabled={!form.name || add.isPending} className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Add</button>
      </div>
      <ul className="space-y-2">
        {fields.map(f => (
          <li key={f.id} className="flex justify-between items-center border rounded px-3 py-2">
            <span>{f.name} <span className="text-gray-500 text-sm">({f.field_type}{f.required ? ', required' : ''})</span></span>
            <button onClick={() => remove.mutate(f.id)} className="text-red-600 hover:text-red-800 text-sm">Remove</button>
          </li>
        ))}
      </ul>
    </div>
  )
}
