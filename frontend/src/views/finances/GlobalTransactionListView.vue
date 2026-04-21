<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useQuery } from '@tanstack/vue-query'
import { financeApi } from '../../api/finance'
import { useFinanceStore } from '../../stores/finance'
import type { Transaction, TransactionCategory } from '../../api/finance'

const router = useRouter()
const financeStore = useFinanceStore()

// Filters
const selectedAccountId = ref<string>('')
const selectedType = ref<string>('')
const selectedCategory = ref<string>('')
const selectedMemberId = ref<string>('')
const dateFrom = ref<string>('')
const dateTo = ref<string>('')
const showReconciled = ref<boolean | null>(null)
const limit = ref(50)
const offset = ref(0)

// Load categories
const { data: categories } = useQuery({
  queryKey: ['categories'],
  queryFn: () => financeApi.listCategories(),
})

// Load all transactions with filters
const { data: transactions, isLoading } = useQuery({
  queryKey: ['allTransactions', selectedAccountId, selectedType, selectedCategory, selectedMemberId, dateFrom, dateTo, showReconciled, limit, offset],
  queryFn: async () => {
    const filters: Record<string, any> = {}
    if (selectedAccountId.value) filters.account_id = selectedAccountId.value
    if (selectedType.value) filters.type = selectedType.value
    if (selectedCategory.value) filters.category = selectedCategory.value
    if (selectedMemberId.value) filters.member_id = selectedMemberId.value
    if (dateFrom.value) filters.date_from = dateFrom.value
    if (dateTo.value) filters.date_to = dateTo.value
    if (showReconciled.value !== null) filters.reconciled = showReconciled.value

    return financeApi.listAllTransactions(limit.value, offset.value, filters)
  },
})

const accounts = computed(() => financeStore.accounts)

function onRowClick(row: Transaction) {
  router.push(`/finances/transactions/${row.id}`)
}

function resetFilters() {
  selectedAccountId.value = ''
  selectedType.value = ''
  selectedCategory.value = ''
  selectedMemberId.value = ''
  dateFrom.value = ''
  dateTo.value = ''
  showReconciled.value = null
  offset.value = 0
}

function goToNewTransaction() {
  router.push('/finances/transactions/new')
}

const transactionTypes = ['Income', 'Expense', 'Transfer', 'Refund']
</script>

<template>
  <div style="padding: 24px; max-width: 1400px; margin: 0 auto">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px">
      <h1 style="margin: 0">All Transactions</h1>
      <el-button type="primary" @click="goToNewTransaction">New Transaction</el-button>
    </div>

    <!-- Filter Bar -->
    <el-card style="margin-bottom: 24px">
      <template #header>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <span>Filters</span>
          <el-button link @click="resetFilters">Reset all</el-button>
        </div>
      </template>

      <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px">
        <el-select v-model="selectedAccountId" placeholder="All accounts" clearable>
          <el-option v-for="account in accounts" :key="account.id" :label="account.name" :value="account.id" />
        </el-select>

        <el-select v-model="selectedType" placeholder="All types" clearable>
          <el-option v-for="type in transactionTypes" :key="type" :label="type" :value="type" />
        </el-select>

        <el-select v-model="selectedCategory" placeholder="All categories" clearable>
          <el-option v-for="cat in categories" :key="cat.id" :label="cat.name" :value="cat.name" />
        </el-select>

        <el-date-picker v-model="dateFrom" type="date" placeholder="From date" style="width: 100%" />
        <el-date-picker v-model="dateTo" type="date" placeholder="To date" style="width: 100%" />

        <el-select v-model="showReconciled" placeholder="All reconciliation states" clearable>
          <el-option label="Reconciled" :value="true" />
          <el-option label="Not reconciled" :value="false" />
        </el-select>
      </div>
    </el-card>

    <!-- Table -->
    <div v-if="isLoading" style="text-align: center; padding: 40px">
      <el-spin />
    </div>
    <el-table v-else :data="transactions ?? []" style="width: 100%" @row-click="onRowClick">
      <el-table-column prop="date" label="Date" width="120" sortable />
      <el-table-column prop="bank_account_id" label="Account" width="120">
        <template #default="{ row }">
          {{ accounts.find(a => a.id === row.bank_account_id)?.name }}
        </template>
      </el-table-column>
      <el-table-column prop="type" label="Type" width="100" />
      <el-table-column prop="category" label="Category" width="120" />
      <el-table-column prop="reference" label="Reference" />
      <el-table-column prop="amount" label="Amount" width="120" align="right">
        <template #default="{ row }">
          {{ row.amount.toFixed(2) }} EUR
        </template>
      </el-table-column>
      <el-table-column prop="reconciled" label="Reconciled" width="100">
        <template #default="{ row }">
          <el-tag :type="row.reconciled ? 'success' : 'warning'">
            {{ row.reconciled ? 'Yes' : 'No' }}
          </el-tag>
        </template>
      </el-table-column>
    </el-table>

    <!-- Pagination -->
    <el-pagination
      v-model:current-page="offset"
      :page-size="limit"
      layout="prev, pager, next"
      :total="(transactions?.length ?? 0) + offset"
      style="margin-top: 16px; text-align: right"
    />
  </div>
</template>
