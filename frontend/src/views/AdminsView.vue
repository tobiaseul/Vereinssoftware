<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { getAdmins, createAdmin, deleteAdmin } from '../api/admins'
import { useAuthStore } from '../stores/auth'
import type { AdminRole, Admin } from '../types'

const auth = useAuthStore()
const qc = useQueryClient()

const form = ref({ username: '', password: '', role: 'Admin' as AdminRole })
const error = ref<string | null>(null)

const { data: admins } = useQuery({ queryKey: ['admins'], queryFn: getAdmins })

const add = useMutation({
  mutationFn: () => createAdmin(form.value),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['admins'] })
    form.value = { username: '', password: '', role: 'Admin' }
    error.value = null
  },
  onError: (err: unknown) => {
    error.value = err instanceof Error ? err.message : 'Failed to add'
  },
})

const remove = useMutation({
  mutationFn: (id: string) => deleteAdmin(id),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['admins'] })
    error.value = null
  },
  onError: (err: unknown) => {
    error.value = err instanceof Error ? err.message : 'Failed to delete'
  },
})

function confirmRemove(admin: Admin) {
  if (confirm(`Remove ${admin.username}?`)) remove.mutate(admin.id)
}
</script>

<template>
  <div style="padding:24px;max-width:600px;margin:0 auto">
    <h1 style="margin-bottom:16px">Admin Users</h1>
    <el-alert v-if="error" :title="error" type="error" :closable="false" style="margin-bottom:12px" />

    <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-bottom:16px">
      <el-input v-model="form.username" placeholder="Username" />
      <el-input v-model="form.password" type="password" placeholder="Password" />
      <el-select v-model="form.role">
        <el-option label="Admin" value="Admin" />
        <el-option label="SuperAdmin" value="SuperAdmin" />
      </el-select>
      <el-button type="primary" :disabled="!form.username || !form.password || add.isPending.value"
        @click="add.mutate()">
        Add Admin
      </el-button>
    </div>

    <el-table :data="admins ?? []" style="width:100%">
      <el-table-column label="Username">
        <template #default="{ row }">
          {{ row.username }}
          <el-tag v-if="row.id === auth.auth?.admin_id" size="small" style="margin-left:6px">you</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="role" label="Role" />
      <el-table-column label="" width="100">
        <template #default="{ row }">
          <el-button v-if="row.id !== auth.auth?.admin_id" type="danger" size="small" text
            :loading="remove.isPending.value"
            @click="confirmRemove(row)">
            Remove
          </el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>
