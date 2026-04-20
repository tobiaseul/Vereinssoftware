<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useFinanceStore } from '../../stores/finance'

const router = useRouter()
const route = useRoute()
const financeStore = useFinanceStore()

const accountId = computed(() => route.params.id as string)

const account = computed(() => {
  return financeStore.accounts.find(a => a.id === accountId.value)
})

const recentTransactions = computed(() => {
  return financeStore.transactions
    .filter(t => t.bank_account_id === accountId.value)
    .slice(0, 5)
})

function goToNewTransaction() {
  router.push(`/finances/accounts/${accountId.value}/transactions/new`)
}

function goToReconciliation() {
  router.push(`/finances/accounts/${accountId.value}/reconciliation`)
}
</script>

<template>
  <div class="account-detail" v-if="account">
    <h1>{{ account.name }}</h1>

    <el-card>
      <div class="account-info">
        <div class="info-item">
          <span class="label">IBAN</span>
          <span class="value">{{ account.iban }}</span>
        </div>
        <div class="info-item">
          <span class="label">Bank Name</span>
          <span class="value">{{ account.bank_name }}</span>
        </div>
        <div class="info-item">
          <span class="label">Balance</span>
          <span class="value">{{ account.balance.toFixed(2) }} EUR</span>
        </div>
        <div class="info-item">
          <span class="label">Status</span>
          <span class="value">{{ account.is_active ? 'Active' : 'Inactive' }}</span>
        </div>
      </div>
    </el-card>

    <h2>Recent Transactions</h2>
    <el-table :data="recentTransactions">
      <el-table-column prop="date" label="Date" />
      <el-table-column prop="type" label="Type" />
      <el-table-column prop="reference" label="Reference" />
      <el-table-column prop="amount" label="Amount">
        <template #default="{ row }">
          {{ row.amount.toFixed(2) }} EUR
        </template>
      </el-table-column>
    </el-table>

    <div class="actions">
      <el-button type="primary" @click="goToNewTransaction">New Transaction</el-button>
      <el-button @click="goToReconciliation">Reconcile</el-button>
    </div>
  </div>
</template>

<style scoped>
.account-detail {
  padding: 20px;
}

.account-info {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  margin-bottom: 20px;
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
