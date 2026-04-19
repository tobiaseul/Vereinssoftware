<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useFinanceStore } from '../../stores/finance'

const router = useRouter()
const financeStore = useFinanceStore()

onMounted(() => {
  financeStore.loadAccounts()
})

const totalBalance = computed(() => {
  return financeStore.accounts.reduce((sum, account) => sum + account.balance, 0)
})

const accounts = computed(() => financeStore.accounts)

function goToNewTransaction() {
  if (accounts.value.length > 0) {
    router.push(`/finances/accounts/${accounts.value[0].id}/transactions/new`)
  }
}

function goToAccounts() {
  router.push('/finances/accounts')
}
</script>

<template>
  <div class="finances-dashboard">
    <h1>Finances Dashboard</h1>

    <div class="summary">
      <el-card>
        <div class="summary-item">
          <span class="label">Total Balance</span>
          <span class="value">{{ totalBalance.toFixed(2) }} EUR</span>
        </div>
      </el-card>

      <el-card>
        <div class="summary-item">
          <span class="label">Accounts</span>
          <span class="value">{{ accounts.length }}</span>
        </div>
      </el-card>
    </div>

    <div class="actions">
      <el-button type="primary" @click="goToNewTransaction">New Transaction</el-button>
      <el-button @click="goToAccounts">View Accounts</el-button>
    </div>
  </div>
</template>

<style scoped>
.finances-dashboard {
  padding: 20px;
}

.summary {
  display: flex;
  gap: 20px;
  margin: 20px 0;
}

.summary-item {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.label {
  font-size: 12px;
  color: #666;
  text-transform: uppercase;
}

.value {
  font-size: 24px;
  font-weight: bold;
  color: #333;
}

.actions {
  margin-top: 20px;
  display: flex;
  gap: 10px;
}
</style>
