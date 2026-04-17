<script setup lang="ts">
import { computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { getFieldDefinitions } from '../api/fieldDefinitions'
import type { Member, MembershipType } from '../types'

const props = defineProps<{ modelValue: Partial<Member>; disabled?: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: Partial<Member>): void }>()

const { data: fields } = useQuery({
  queryKey: ['field-definitions'],
  queryFn: getFieldDefinitions,
})

const cf = computed(() => (props.modelValue.custom_fields as Record<string, unknown>) ?? {})

function set(key: keyof Member, val: unknown) {
  emit('update:modelValue', { ...props.modelValue, [key]: val })
}

function setCf(name: string, val: unknown) {
  set('custom_fields', { ...cf.value, [name]: val })
}
</script>

<template>
  <el-form label-position="top">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px">
      <el-form-item label="First Name *">
        <el-input :model-value="modelValue.first_name ?? ''"
          @update:model-value="set('first_name', $event)" :disabled="disabled" />
      </el-form-item>
      <el-form-item label="Last Name *">
        <el-input :model-value="modelValue.last_name ?? ''"
          @update:model-value="set('last_name', $event)" :disabled="disabled" />
      </el-form-item>
      <el-form-item label="Email">
        <el-input type="email" :model-value="modelValue.email ?? ''"
          @update:model-value="set('email', $event || null)" :disabled="disabled" />
      </el-form-item>
      <el-form-item label="Phone">
        <el-input :model-value="modelValue.phone ?? ''"
          @update:model-value="set('phone', $event || null)" :disabled="disabled" />
      </el-form-item>
    </div>

    <el-form-item label="Membership Type *">
      <el-select :model-value="modelValue.membership_type ?? ''"
        @update:model-value="set('membership_type', $event as MembershipType)" :disabled="disabled">
        <el-option label="Aktiv" value="Aktiv" />
        <el-option label="Passiv" value="Passiv" />
        <el-option label="Ehrenmitglied" value="Ehrenmitglied" />
      </el-select>
    </el-form-item>

    <el-form-item label="Notes">
      <el-input type="textarea" :rows="3" :model-value="modelValue.notes ?? ''"
        @update:model-value="set('notes', $event || null)" :disabled="disabled" />
    </el-form-item>

    <template v-if="fields && fields.length > 0">
      <el-divider />
      <h3 style="margin-bottom:12px">Custom Fields</h3>
      <el-form-item v-for="f in fields" :key="f.id" :label="f.name + (f.required ? ' *' : '')">
        <el-select v-if="f.field_type === 'enum'"
          :model-value="cf[f.name] as string ?? ''"
          @update:model-value="setCf(f.name, $event || null)"
          :disabled="disabled" clearable>
          <el-option v-for="opt in f.options" :key="opt.id" :label="opt.value" :value="opt.value" />
        </el-select>
        <el-checkbox v-else-if="f.field_type === 'boolean'"
          :model-value="!!cf[f.name]"
          @update:model-value="setCf(f.name, $event)"
          :disabled="disabled" />
        <el-input v-else
          :type="f.field_type === 'number' ? 'number' : f.field_type === 'date' ? 'date' : 'text'"
          :model-value="cf[f.name] as string ?? ''"
          @update:model-value="setCf(f.name, $event || null)"
          :disabled="disabled" />
      </el-form-item>
    </template>
  </el-form>
</template>
