<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useFinanceStore } from '../../stores/finance'

const router = useRouter()
const route = useRoute()
const financeStore = useFinanceStore()

const accountId = computed(() => route.params.id as string)
const selectedCategory = ref('')
const selectedType = ref('')
const dateRange = ref('')

const transactions = computed(() => {
  return financeStore.transactions.filter(t => t.accountId === accountId.value)
})

const filteredTransactions = computed(() => {
  return transactions.value.filter(t => {
    if (selectedCategory.value && t.category !== selectedCategory.value) return false
    if (selectedType.value && t.type !== selectedType.value) return false
    return true
  })
})

function handleRowClick(row: any) {
  router.push(`/finances/transactions/${row.id}`)
}

function exportCsv() {
  // TODO: Implement CSV export
  ElMessage.info('CSV export coming soon')
}
</script>

<template>
  <div class="transaction-list">
    <h1>Transactions</h1>

    <div class="filters">
      <el-select v-model="selectedCategory" placeholder="Filter by Category">
        <el-option label="All Categories" value="" />
        <el-option label="Income" value="income" />
        <el-option label="Expense" value="expense" />
      </el-select>

      <el-select v-model="selectedType" placeholder="Filter by Type">
        <el-option label="All Types" value="" />
        <el-option label="Transfer" value="transfer" />
        <el-option label="Payment" value="payment" />
      </el-select>

      <el-button type="primary" @click="exportCsv">Export CSV</el-button>
    </div>

    <el-table :data="filteredTransactions" @row-click="handleRowClick">
      <el-table-column prop="date" label="Date" />
      <el-table-column prop="type" label="Type" />
      <el-table-column prop="category" label="Category" />
      <el-table-column prop="reference" label="Reference" />
      <el-table-column prop="amount" label="Amount">
        <template #default="{ row }">
          {{ row.amount.toFixed(2) }} EUR
        </template>
      </el-table-column>
      <el-table-column prop="reconciled" label="Reconciled">
        <template #default="{ row }">
          <el-tag :type="row.reconciled ? 'success' : 'info'">
            {{ row.reconciled ? 'Yes' : 'No' }}
          </el-tag>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<style scoped>
.transaction-list {
  padding: 20px;
}

.filters {
  display: flex;
  gap: 10px;
  margin: 20px 0;
}

.filters :deep(.el-select) {
  width: 200px;
}
</style>
