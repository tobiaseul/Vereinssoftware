<script setup lang="ts">
import { ref, computed } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { getAdminFinanceRoles, assignFinanceRole, removeFinanceRole } from '../api/admins'
import { ElMessage } from 'element-plus'

const props = defineProps<{
  adminId: string
}>()

const qc = useQueryClient()
const selectedRole = ref('')
const showForm = ref(false)

const { data: roles } = useQuery({
  queryKey: ['financeRoles', props.adminId],
  queryFn: () => getAdminFinanceRoles(props.adminId),
})

const assign = useMutation({
  mutationFn: () => {
    console.log('Assigning role:', selectedRole.value, 'to admin:', props.adminId)
    return assignFinanceRole(props.adminId, selectedRole.value)
  },
  onSuccess: () => {
    console.log('Role assigned successfully')
    qc.invalidateQueries({ queryKey: ['financeRoles', props.adminId] })
    selectedRole.value = ''
    showForm.value = false
    ElMessage.success('Role assigned')
  },
  onError: (err: unknown) => {
    console.error('Error assigning role:', err)
    const message = err instanceof Error ? err.message : 'Failed to assign role'
    ElMessage.error(message)
  },
})

const removeRole = useMutation({
  mutationFn: (roleName: string) => removeFinanceRole(props.adminId, roleName),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['financeRoles', props.adminId] })
    ElMessage.success('Role removed')
  },
  onError: (err: unknown) => {
    console.error('Error removing role:', err)
    const message = err instanceof Error ? err.message : 'Failed to remove role'
    ElMessage.error(message)
  },
})

const isAssigning = computed(() => assign.isPending.value === true)
const isRemoving = computed(() => removeRole.isPending.value === true)

function handleAssign() {
  if (!selectedRole.value) {
    ElMessage.warning('Please select a role')
    return
  }
  assign.mutate()
}
</script>

<template>
  <div style="display: flex; gap: 8px; align-items: center;">
    <!-- Display current roles -->
    <div style="display: flex; gap: 4px; flex-wrap: wrap; flex: 1;">
      <el-tag
        v-for="role in roles"
        :key="role.id"
        closable
        size="small"
        :disabled="isRemoving"
        @close="removeRole.mutate(role.name)"
      >
        {{ role.name }}
      </el-tag>
      <span v-if="!roles?.length" style="color: #999; font-size: 12px;">None</span>
    </div>

    <!-- Toggle form button -->
    <el-button
      v-if="!showForm"
      type="primary"
      link
      size="small"
      @click="showForm = true"
    >
      + Add
    </el-button>

    <!-- Form to add role -->
    <div v-if="showForm" style="display: flex; gap: 8px; align-items: center;">
      <el-select v-model="selectedRole" placeholder="Select role" size="small" style="width: 150px;">
        <el-option label="Treasurer" value="Treasurer" />
        <el-option label="Finance Officer" value="Finance Officer" />
      </el-select>
      <el-button
        type="primary"
        size="small"
        @click="handleAssign"
        :loading="isAssigning"
        :disabled="isAssigning"
      >
        Add
      </el-button>
      <el-button
        size="small"
        @click="showForm = false"
      >
        Cancel
      </el-button>
    </div>
  </div>
</template>
