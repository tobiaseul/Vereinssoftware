<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useFinanceStore } from '../../stores/finance'

const router = useRouter()
const route = useRoute()
const financeStore = useFinanceStore()

const transactionId = computed(() => route.params.id as string)

const transaction = computed(() => {
  return financeStore.transactions.find(t => t.id === transactionId.value)
})

function downloadReceipt() {
  // TODO: Implement receipt download
  ElMessage.info('Receipt download coming soon')
}

function deleteTransaction() {
  ElMessageBox.confirm(
    'Delete this transaction?',
    'Warning',
    {
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
      type: 'warning',
    }
  )
    .then(() => {
      // TODO: Implement deletion
      ElMessage.success('Transaction deleted')
      router.back()
    })
    .catch(() => {
      ElMessage.info('Delete cancelled')
    })
}
</script>

<template>
  <div class="transaction-detail" v-if="transaction">
    <h1>{{ transaction.reference }}</h1>

    <el-card>
      <div class="transaction-info">
        <div class="info-item">
          <span class="label">Type</span>
          <span class="value">{{ transaction.type }}</span>
        </div>
        <div class="info-item">
          <span class="label">Amount</span>
          <span class="value">{{ transaction.amount.toFixed(2) }} EUR</span>
        </div>
        <div class="info-item">
          <span class="label">Date</span>
          <span class="value">{{ transaction.date }}</span>
        </div>
        <div class="info-item">
          <span class="label">Category</span>
          <span class="value">{{ transaction.category }}</span>
        </div>
      </div>
    </el-card>

    <div class="actions">
      <el-button type="primary" @click="downloadReceipt">Download Receipt</el-button>
      <el-button type="danger" @click="deleteTransaction">Delete</el-button>
    </div>
  </div>
</template>

<style scoped>
.transaction-detail {
  padding: 20px;
}

.transaction-info {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.label {
  font-size: 12px;
  color: #666;
  text-transform: uppercase;
}

.value {
  font-size: 16px;
  font-weight: 500;
}

.actions {
  margin-top: 20px;
  display: flex;
  gap: 10px;
}
</style>
