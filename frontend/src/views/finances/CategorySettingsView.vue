<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { financeApi } from '../../api/finance'
import { ElMessage } from 'element-plus'

const qc = useQueryClient()
const newCategoryName = ref('')
const error = ref<string | null>(null)

const { data: categories } = useQuery({
  queryKey: ['categories'],
  queryFn: () => financeApi.listCategories(),
})

const createMutation = useMutation({
  mutationFn: () => financeApi.createCategory(newCategoryName.value),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['categories'] })
    newCategoryName.value = ''
    error.value = null
    ElMessage.success('Category created')
  },
  onError: (err: unknown) => {
    const message = err instanceof Error ? err.message : 'Failed to create category'
    error.value = message
    ElMessage.error(message)
  },
})

const deleteMutation = useMutation({
  mutationFn: (id: string) => financeApi.deleteCategory(id),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['categories'] })
    ElMessage.success('Category deleted')
  },
  onError: (err: unknown) => {
    const message = err instanceof Error ? err.message : 'Failed to delete category'
    error.value = message
    ElMessage.error(message)
  },
})

function handleCreate() {
  if (!newCategoryName.value.trim()) {
    error.value = 'Category name cannot be empty'
    ElMessage.warning(error.value)
    return
  }
  createMutation.mutate()
}

function handleDelete(id: string) {
  if (confirm('Are you sure?')) {
    deleteMutation.mutate(id)
  }
}
</script>

<template>
  <div style="padding: 24px; max-width: 600px; margin: 0 auto">
    <h1 style="margin-bottom: 16px">Transaction Categories</h1>

    <el-alert v-if="error" :title="error" type="error" :closable="true" @close="error = null" style="margin-bottom: 12px" />

    <div style="display: flex; gap: 8px; margin-bottom: 24px">
      <el-input
        v-model="newCategoryName"
        placeholder="New category name"
        style="flex: 1"
        @keyup.enter="handleCreate"
      />
      <el-button
        type="primary"
        :loading="createMutation.isPending"
        @click="handleCreate"
      >
        Add
      </el-button>
    </div>

    <el-table :data="categories ?? []" style="width: 100%">
      <el-table-column prop="name" label="Name" />
      <el-table-column label="" width="100">
        <template #default="{ row }">
          <el-button
            type="danger"
            size="small"
            text
            :loading="deleteMutation.isPending"
            @click="handleDelete(row.id)"
          >
            Delete
          </el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>
