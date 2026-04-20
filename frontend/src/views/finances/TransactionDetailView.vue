<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useFinanceStore } from '../../stores/finance'
import { financeApi } from '../../api/finance'

const router = useRouter()
const route = useRoute()
const financeStore = useFinanceStore()

const transactionId = computed(() => route.params.id as string)
const isLoading = ref(false)

const transaction = computed(() => {
  return financeStore.transactions.find(t => t.id === transactionId.value)
})

onMounted(() => {
  // Transaction should already be loaded from parent view
  // If not found, could add loading logic here
})

async function downloadReceipt() {
  if (!transaction.value?.receipt_reference) {
    ElMessage.info('No receipt available for this transaction')
    return
  }

  isLoading.value = true
  try {
    const blob = await financeApi.downloadReceipt(transactionId.value, transaction.value.receipt_reference)
    const url = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `receipt-${transaction.value.reference}.pdf`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)
  } catch (err) {
    ElMessage.error(err instanceof Error ? err.message : 'Failed to download receipt')
  } finally {
    isLoading.value = false
  }
}

async function deleteTransaction() {
  ElMessageBox.confirm(
    'Delete this transaction? This action cannot be undone.',
    'Warning',
    {
      confirmButtonText: 'Delete',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  )
    .then(async () => {
      isLoading.value = true
      try {
        await financeStore.softDeleteTransaction(transactionId.value)
        ElMessage.success('Transaction deleted')
        router.back()
      } catch (err) {
        ElMessage.error(err instanceof Error ? err.message : 'Failed to delete transaction')
        isLoading.value = false
      }
    })
    .catch(() => {
      ElMessage.info('Delete cancelled')
    })
}

function editTransaction() {
  if (transaction.value) {
    router.push({
      name: 'transaction-form',
      params: {
        id: transaction.value.bank_account_id,
        transactionId: transactionId.value,
      },
    })
  }
}
</script>

<template>
  <div class="transaction-detail" v-if="transaction">
    <h1>{{ transaction.reference }}</h1>

    <el-card>
      <el-descriptions :column="2" border>
        <el-descriptions-item label="Type">{{ transaction.type }}</el-descriptions-item>
        <el-descriptions-item label="Amount">{{ transaction.amount.toFixed(2) }} EUR</el-descriptions-item>
        <el-descriptions-item label="Date">{{ transaction.date }}</el-descriptions-item>
        <el-descriptions-item label="Category">{{ transaction.category }}</el-descriptions-item>
        <el-descriptions-item label="Reference">{{ transaction.reference }}</el-descriptions-item>
        <el-descriptions-item label="Reconciled">
          <el-tag :type="transaction.reconciled ? 'success' : 'info'">
            {{ transaction.reconciled ? 'Yes' : 'No' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item v-if="transaction.description" label="Description" :span="2">
          {{ transaction.description }}
        </el-descriptions-item>
      </el-descriptions>

      <div v-if="transaction.receipt_reference" class="receipt-section">
        <h3>Receipt</h3>
        <p>File: {{ transaction.receipt_reference }}</p>
        <el-button type="primary" size="small" :loading="isLoading" @click="downloadReceipt">
          Download Receipt
        </el-button>
      </div>
    </el-card>

    <div class="actions">
      <el-button type="primary" @click="editTransaction">Edit</el-button>
      <el-button type="danger" :loading="isLoading" @click="deleteTransaction">Delete</el-button>
    </div>
  </div>
</template>

<style scoped>
.transaction-detail {
  padding: 20px;
}

.receipt-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #ebeef5;
}

.receipt-section h3 {
  margin-top: 0;
  margin-bottom: 10px;
  font-size: 16px;
}

.receipt-section p {
  margin: 0 0 10px 0;
  color: #606266;
  font-size: 14px;
}

.actions {
  margin-top: 20px;
  display: flex;
  gap: 10px;
}
</style>
