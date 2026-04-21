<script setup lang="ts">
import { ref, computed, onMounted, type Ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useQuery } from '@tanstack/vue-query'
import { useFinanceStore } from '../../stores/finance'
import { financeApi, type Transaction } from '../../api/finance'
import { getMembers } from '../../api/members'
import type { Member } from '../../types'

interface Props {
  accountId?: string
  transaction?: Transaction
  isEditMode?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isEditMode: false,
})

const router = useRouter()
const financeStore = useFinanceStore()

// Form state
const form = ref({
  accountId: '',
  type: '',
  amount: undefined as number | undefined,
  date: '',
  memberId: '',
  category: '',
  reference: '',
  description: '',
})

const selectedFile = ref<File | null>(null)
const isLoading = ref(false)

// Query for accounts
const { data: accountsQuery } = useQuery({
  queryKey: ['accounts'],
  queryFn: () => financeApi.listAccounts(),
})

// Query for members
const { data: membersQuery } = useQuery({
  queryKey: ['members'],
  queryFn: () => getMembers(),
})

// Query for categories
const { data: categoriesQuery } = useQuery({
  queryKey: ['categories'],
  queryFn: () => financeApi.listCategories(),
})

const accounts = computed(() => accountsQuery.value ?? [])
const members = computed(() => membersQuery.value ?? [])
const categories = computed(() => categoriesQuery.value ?? [])

// Initialize form with transaction data if in edit mode
onMounted(() => {
  if (props.accountId) {
    form.value.accountId = props.accountId
  }
  if (props.transaction && props.isEditMode) {
    form.value = {
      accountId: props.transaction.bank_account_id,
      type: props.transaction.type,
      amount: props.transaction.amount,
      date: props.transaction.date,
      memberId: props.transaction.member_id || '',
      category: props.transaction.category,
      reference: props.transaction.reference,
      description: props.transaction.description || '',
    }
  }
})

function handleFileSelect(file: File) {
  // Validate file
  const maxSize = 10 * 1024 * 1024 // 10MB
  const allowedTypes = ['image/jpeg', 'image/png', 'application/pdf']
  const allowedExtensions = ['.jpg', '.jpeg', '.png', '.pdf']

  const fileExtension = '.' + file.name.split('.').pop()?.toLowerCase()

  if (file.size > maxSize) {
    ElMessage.error('File size must not exceed 10MB')
    return false
  }

  if (!allowedTypes.includes(file.type) && !allowedExtensions.includes(fileExtension)) {
    ElMessage.error('Only JPG, PNG, and PDF files are allowed')
    return false
  }

  selectedFile.value = file
  return false // Prevent default upload behavior
}

async function handleSubmit() {
  // Validate required fields
  if (!form.value.accountId || !form.value.type || form.value.amount === undefined || !form.value.date || !form.value.category || !form.value.reference) {
    ElMessage.error('Please fill in all required fields')
    return
  }

  isLoading.value = true
  try {
    let transaction: Transaction

    if (props.isEditMode && props.transaction) {
      // Update transaction
      transaction = await financeStore.updateTransaction(
        props.transaction.id,
        props.transaction.version,
        form.value.amount,
        form.value.date,
        form.value.category,
        form.value.reference,
        form.value.description || undefined
      )
    } else {
      // Create new transaction
      transaction = await financeStore.createTransaction(
        form.value.accountId,
        form.value.type,
        form.value.amount,
        form.value.date,
        form.value.category,
        form.value.reference,
        form.value.memberId || undefined,
        form.value.description || undefined
      )
    }

    // Upload receipt if selected
    if (selectedFile.value) {
      try {
        await financeStore.uploadReceipt(transaction.id, selectedFile.value)
      } catch (err) {
        ElMessage.warning('Transaction created but receipt upload failed')
      }
    }

    ElMessage.success(props.isEditMode ? 'Transaction updated' : 'Transaction created')
    router.back()
  } catch (err) {
    ElMessage.error(err instanceof Error ? err.message : 'Failed to save transaction')
  } finally {
    isLoading.value = false
  }
}

function handleCancel() {
  router.back()
}

const memberOptions = computed(() =>
  (members.value as Member[]).map(m => ({
    label: `${m.first_name} ${m.last_name}`,
    value: m.id,
  }))
)
</script>

<template>
  <el-form label-position="top" class="transaction-form">
    <div class="form-grid">
      <el-form-item label="Account *" required>
        <el-select v-model="form.accountId" placeholder="Select account">
          <el-option v-for="account in accounts" :key="account.id" :label="account.name" :value="account.id" />
        </el-select>
      </el-form-item>

      <el-form-item label="Type *" required>
        <el-select v-model="form.type" placeholder="Select transaction type">
          <el-option label="Income" value="Income" />
          <el-option label="Expense" value="Expense" />
          <el-option label="Transfer" value="Transfer" />
          <el-option label="Refund" value="Refund" />
        </el-select>
      </el-form-item>

      <el-form-item label="Amount *" required>
        <el-input-number v-model="form.amount" :precision="2" :step="0.01" placeholder="0.00" />
      </el-form-item>

      <el-form-item label="Date *" required>
        <el-date-picker v-model="form.date" type="date" placeholder="Select date" value-format="YYYY-MM-DD" />
      </el-form-item>

      <el-form-item label="Member (optional)">
        <el-select v-model="form.memberId" placeholder="Search members..." clearable filterable>
          <el-option v-for="option in memberOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
      </el-form-item>

      <el-form-item label="Category *" required>
        <el-select v-model="form.category" placeholder="Select category">
          <el-option v-for="cat in categories" :key="cat.id" :label="cat.name" :value="cat.name" />
        </el-select>
      </el-form-item>

      <el-form-item label="Reference *" required>
        <el-input v-model="form.reference" placeholder="e.g., Invoice #123 or Check #456" />
      </el-form-item>
    </div>

    <el-form-item label="Description (optional)">
      <el-input v-model="form.description" type="textarea" :rows="3" placeholder="Additional notes..." />
    </el-form-item>

    <el-form-item label="Receipt (optional)">
      <el-upload
        drag
        action="#"
        :auto-upload="false"
        :on-change="handleFileSelect"
        accept=".jpg,.jpeg,.png,.pdf"
      >
        <el-icon class="el-icon--upload"><i-carbon-cloud-upload /></el-icon>
        <div class="el-upload__text">
          Drop file here or <em>click to upload</em>
        </div>
        <template #tip>
          <div class="el-upload__tip">
            JPG, PNG, or PDF files up to 10MB
          </div>
        </template>
      </el-upload>
      <div v-if="selectedFile" class="selected-file">
        <p>Selected: {{ selectedFile.name }}</p>
      </div>
    </el-form-item>

    <el-form-item>
      <el-button type="primary" :loading="isLoading" @click="handleSubmit">
        {{ isEditMode ? 'Update' : 'Create' }} Transaction
      </el-button>
      <el-button @click="handleCancel">
        Cancel
      </el-button>
    </el-form-item>
  </el-form>
</template>

<style scoped>
.transaction-form {
  max-width: 600px;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.selected-file {
  margin-top: 10px;
  padding: 10px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.selected-file p {
  margin: 0;
  font-size: 14px;
  color: #606266;
}
</style>
