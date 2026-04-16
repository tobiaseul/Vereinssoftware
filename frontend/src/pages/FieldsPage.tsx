// frontend/src/pages/FieldsPage.tsx
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  getFieldDefinitions, createFieldDefinition, updateFieldDefinition, deleteFieldDefinition,
  addFieldOption, updateFieldOption, deleteFieldOption,
} from '../api/fieldDefinitions'
import type { FieldDefinition, FieldType } from '../types'

function OptionsList({ field }: { field: FieldDefinition }) {
  const qc = useQueryClient()
  const [newValue, setNewValue] = useState('')
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editValue, setEditValue] = useState('')
  const [error, setError] = useState<string | null>(null)

  const add = useMutation({
    mutationFn: () => addFieldOption(field.id, { value: newValue }),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setNewValue(''); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to add option'),
  })

  const update = useMutation({
    mutationFn: ({ optionId, value }: { optionId: string; value: string }) =>
      updateFieldOption(field.id, optionId, { value }),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setEditingId(null); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to update option'),
  })

  const remove = useMutation({
    mutationFn: (optionId: string) => deleteFieldOption(field.id, optionId),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to delete option'),
  })

  return (
    <div className="mt-2 ml-4 pl-4 border-l border-gray-200 space-y-1">
      {error && <p className="text-red-600 text-xs">{error}</p>}
      {field.options.map(opt => (
        <div key={opt.id} className="flex items-center gap-2">
          {editingId === opt.id ? (
            <>
              <input className="border rounded px-2 py-1 text-sm flex-1"
                value={editValue} onChange={e => setEditValue(e.target.value)} />
              <button onClick={() => update.mutate({ optionId: opt.id, value: editValue })}
                disabled={!editValue || update.isPending}
                className="text-green-600 text-sm hover:text-green-800 disabled:opacity-50">Save</button>
              <button onClick={() => setEditingId(null)}
                className="text-gray-500 text-sm hover:text-gray-700">Cancel</button>
            </>
          ) : (
            <>
              <span className="text-sm flex-1">{opt.value}</span>
              <button onClick={() => { setEditingId(opt.id); setEditValue(opt.value) }}
                className="text-blue-600 text-sm hover:text-blue-800">Edit</button>
              <button onClick={() => { if (confirm(`Remove option "${opt.value}"?`)) remove.mutate(opt.id) }}
                className="text-red-600 text-sm hover:text-red-800">Remove</button>
            </>
          )}
        </div>
      ))}
      <div className="flex gap-2 mt-2">
        <input className="border rounded px-2 py-1 text-sm flex-1" placeholder="New option..."
          value={newValue} onChange={e => setNewValue(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter' && newValue) add.mutate() }} />
        <button onClick={() => add.mutate()} disabled={!newValue || add.isPending}
          className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50">Add</button>
      </div>
    </div>
  )
}

export function FieldsPage() {
  const qc = useQueryClient()
  const [form, setForm] = useState({ name: '', field_type: 'text' as FieldType, required: false })
  const [error, setError] = useState<string | null>(null)
  const [deletingId, setDeletingId] = useState<string | null>(null)
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editForm, setEditForm] = useState({ name: '', required: false })
  const [expandedId, setExpandedId] = useState<string | null>(null)

  const { data: fields = [] } = useQuery({ queryKey: ['field-definitions'], queryFn: getFieldDefinitions })

  const add = useMutation({
    mutationFn: () => createFieldDefinition(form),
    onSuccess: (newField) => {
      qc.invalidateQueries({ queryKey: ['field-definitions'] })
      setForm({ name: '', field_type: 'text', required: false })
      setError(null)
      if (newField.field_type === 'enum') setExpandedId(newField.id)
    },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to add field'),
  })

  const update = useMutation({
    mutationFn: ({ id, ...data }: { id: string; name: string; required: boolean }) =>
      updateFieldDefinition(id, data),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setEditingId(null); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to update field'),
  })

  const remove = useMutation({
    mutationFn: (id: string) => deleteFieldDefinition(id),
    onMutate: (id) => setDeletingId(id),
    onSettled: () => setDeletingId(null),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to delete field'),
  })

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-4">Custom Fields</h1>
      {error && <p className="text-red-600 text-sm mb-2">{error}</p>}

      <div className="flex gap-2 mb-4">
        <input className="border rounded px-3 py-2 flex-1" placeholder="Field name..."
          value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} />
        <select className="border rounded px-3 py-2" value={form.field_type}
          onChange={e => setForm(f => ({ ...f, field_type: e.target.value as FieldType }))}>
          <option value="text">Text</option>
          <option value="number">Number</option>
          <option value="date">Date</option>
          <option value="boolean">Boolean</option>
          <option value="enum">Dropdown</option>
        </select>
        <label className="flex items-center gap-1 text-sm">
          <input type="checkbox" checked={form.required}
            onChange={e => setForm(f => ({ ...f, required: e.target.checked }))} /> Required
        </label>
        <button onClick={() => add.mutate()} disabled={!form.name || add.isPending}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Add</button>
      </div>

      <ul className="space-y-2">
        {fields.map(f => (
          <li key={f.id} className="border rounded px-3 py-2">
            {editingId === f.id ? (
              <div className="flex gap-2 items-center">
                <input className="border rounded px-2 py-1 flex-1" value={editForm.name}
                  onChange={e => setEditForm(ef => ({ ...ef, name: e.target.value }))} />
                <label className="flex items-center gap-1 text-sm">
                  <input type="checkbox" checked={editForm.required}
                    onChange={e => setEditForm(ef => ({ ...ef, required: e.target.checked }))} /> Required
                </label>
                <button onClick={() => update.mutate({ id: f.id, ...editForm })}
                  disabled={!editForm.name || update.isPending}
                  className="px-2 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 disabled:opacity-50">Save</button>
                <button onClick={() => setEditingId(null)}
                  className="text-sm text-gray-500 hover:text-gray-700">Cancel</button>
              </div>
            ) : (
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2">
                  {f.field_type === 'enum' && (
                    <button onClick={() => setExpandedId(expandedId === f.id ? null : f.id)}
                      className="text-gray-400 hover:text-gray-600 text-xs w-4">
                      {expandedId === f.id ? '▼' : '▶'}
                    </button>
                  )}
                  <span>
                    {f.name}
                    <span className="text-gray-500 text-sm ml-1">
                      ({f.field_type === 'enum' ? `dropdown, ${f.options.length} options` : f.field_type}
                      {f.required ? ', required' : ''})
                    </span>
                  </span>
                </div>
                <div className="flex gap-2">
                  <button onClick={() => { setEditingId(f.id); setEditForm({ name: f.name, required: f.required }) }}
                    className="text-blue-600 hover:text-blue-800 text-sm">Edit</button>
                  <button disabled={deletingId === f.id}
                    onClick={() => { if (confirm(`Remove "${f.name}" and all its options?`)) remove.mutate(f.id) }}
                    className="text-red-600 hover:text-red-800 text-sm disabled:opacity-50">Remove</button>
                </div>
              </div>
            )}
            {expandedId === f.id && f.field_type === 'enum' && <OptionsList field={f} />}
          </li>
        ))}
      </ul>
    </div>
  )
}
