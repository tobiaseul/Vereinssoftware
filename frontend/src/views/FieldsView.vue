<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  getFieldDefinitions, createFieldDefinition, updateFieldDefinition, deleteFieldDefinition,
  addFieldOption, updateFieldOption, deleteFieldOption,
} from '../api/fieldDefinitions'
import type { FieldDefinition, FieldType } from '../types'

const qc = useQueryClient()
const error = ref<string | null>(null)

// Add form
const form = ref({ name: '', field_type: 'text' as FieldType, required: false })

// Edit state
const editingId = ref<string | null>(null)
const editForm = ref({ name: '', required: false })

// Expanded enum options
const expandedId = ref<string | null>(null)

// Option edit state
const editingOptionId = ref<string | null>(null)
const editOptionValue = ref('')
const newOptionValue = ref('')

const { data: fields } = useQuery({ queryKey: ['field-definitions'], queryFn: getFieldDefinitions })

const addField = useMutation({
  mutationFn: () => createFieldDefinition(form.value),
  onSuccess: (newField) => {
    qc.invalidateQueries({ queryKey: ['field-definitions'] })
    form.value = { name: '', field_type: 'text', required: false }
    error.value = null
    if (newField.field_type === 'enum') expandedId.value = newField.id
  },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to add field' },
})

const updateField = useMutation({
  mutationFn: ({ id, ...data }: { id: string; name: string; required: boolean }) =>
    updateFieldDefinition(id, data),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['field-definitions'] })
    editingId.value = null
    error.value = null
  },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to update field' },
})

const removeField = useMutation({
  mutationFn: (id: string) => deleteFieldDefinition(id),
  onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); error.value = null },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to delete field' },
})

function startEdit(f: FieldDefinition) {
  editingId.value = f.id
  editForm.value = { name: f.name, required: f.required }
}

function confirmRemoveField(f: FieldDefinition) {
  if (confirm(`Remove '${f.name}' and all its options?`)) removeField.mutate(f.id)
}

const addOption = useMutation({
  mutationFn: ({ fieldId, value }: { fieldId: string; value: string }) =>
    addFieldOption(fieldId, { value }),
  onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); newOptionValue.value = ''; error.value = null },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to add option' },
})

const updateOption = useMutation({
  mutationFn: ({ fieldId, optionId, value }: { fieldId: string; optionId: string; value: string }) =>
    updateFieldOption(fieldId, optionId, { value }),
  onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); editingOptionId.value = null; error.value = null },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to update option' },
})

const removeOption = useMutation({
  mutationFn: ({ fieldId, optionId }: { fieldId: string; optionId: string }) =>
    deleteFieldOption(fieldId, optionId),
  onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); error.value = null },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to delete option' },
})

function startEditOption(optId: string, optValue: string) {
  editingOptionId.value = optId
  editOptionValue.value = optValue
}

function confirmRemoveOption(fieldId: string, optId: string, optValue: string) {
  if (confirm(`Remove option '${optValue}'?`)) removeOption.mutate({ fieldId, optionId: optId })
}

function submitAddOption(fieldId: string) {
  if (newOptionValue.value) addOption.mutate({ fieldId, value: newOptionValue.value })
}
</script>

<template>
  <div style="padding:24px;max-width:640px;margin:0 auto">
    <h1 style="margin-bottom:16px">Custom Fields</h1>
    <el-alert v-if="error" :title="error" type="error" :closable="false" style="margin-bottom:12px" />

    <!-- Add field form -->
    <div style="display:flex;gap:8px;margin-bottom:16px;align-items:center">
      <el-input v-model="form.name" placeholder="Field name..." style="flex:1" />
      <el-select v-model="form.field_type" style="width:140px">
        <el-option label="Text" value="text" />
        <el-option label="Number" value="number" />
        <el-option label="Date" value="date" />
        <el-option label="Boolean" value="boolean" />
        <el-option label="Dropdown" value="enum" />
      </el-select>
      <el-checkbox v-model="form.required">Required</el-checkbox>
      <el-button type="primary" :disabled="!form.name || addField.isPending.value" @click="addField.mutate()">
        Add
      </el-button>
    </div>

    <!-- Field list -->
    <div v-for="f in fields ?? []" :key="f.id" style="border:1px solid #e4e7ed;border-radius:4px;margin-bottom:8px;padding:12px">
      <!-- Edit row -->
      <template v-if="editingId === f.id">
        <div style="display:flex;gap:8px;align-items:center">
          <el-input v-model="editForm.name" style="flex:1" />
          <el-checkbox v-model="editForm.required">Required</el-checkbox>
          <el-button type="success" size="small"
            :disabled="!editForm.name || updateField.isPending.value"
            @click="updateField.mutate({ id: f.id, ...editForm })">Save</el-button>
          <el-button size="small" @click="editingId = null">Cancel</el-button>
        </div>
      </template>

      <!-- Display row -->
      <template v-else>
        <div style="display:flex;justify-content:space-between;align-items:center">
          <div style="display:flex;align-items:center;gap:8px">
            <el-button v-if="f.field_type === 'enum'" text size="small"
              @click="expandedId = expandedId === f.id ? null : f.id">
              {{ expandedId === f.id ? '▼' : '▶' }}
            </el-button>
            <span>
              {{ f.name }}
              <span style="color:#909399;font-size:13px">
                ({{ f.field_type === 'enum' ? `dropdown, ${f.options.length} options` : f.field_type }}{{ f.required ? ', required' : '' }})
              </span>
            </span>
          </div>
          <div style="display:flex;gap:8px">
            <el-button type="primary" size="small" text @click="startEdit(f)">Edit</el-button>
            <el-button type="danger" size="small" text :loading="removeField.isPending.value"
              @click="confirmRemoveField(f)">
              Remove
            </el-button>
          </div>
        </div>
      </template>

      <!-- Enum options panel -->
      <div v-if="expandedId === f.id && f.field_type === 'enum'"
        style="margin-top:12px;padding-left:16px;border-left:2px solid #e4e7ed">
        <div v-for="opt in f.options" :key="opt.id" style="display:flex;align-items:center;gap:8px;margin-bottom:6px">
          <template v-if="editingOptionId === opt.id">
            <el-input v-model="editOptionValue" size="small" style="flex:1" />
            <el-button size="small" type="success" :disabled="!editOptionValue || updateOption.isPending.value"
              @click="updateOption.mutate({ fieldId: f.id, optionId: opt.id, value: editOptionValue })">Save</el-button>
            <el-button size="small" @click="editingOptionId = null">Cancel</el-button>
          </template>
          <template v-else>
            <span style="flex:1;font-size:14px">{{ opt.value }}</span>
            <el-button size="small" text type="primary"
              @click="startEditOption(opt.id, opt.value)">Edit</el-button>
            <el-button size="small" text type="danger"
              @click="confirmRemoveOption(f.id, opt.id, opt.value)">
              Remove
            </el-button>
          </template>
        </div>
        <div style="display:flex;gap:8px;margin-top:8px">
          <el-input v-model="newOptionValue" size="small" placeholder="New option..."
            style="flex:1" @keydown.enter="submitAddOption(f.id)" />
          <el-button size="small" type="primary" :disabled="!newOptionValue || addOption.isPending.value"
            @click="submitAddOption(f.id)">Add</el-button>
        </div>
      </div>
    </div>
  </div>
</template>
