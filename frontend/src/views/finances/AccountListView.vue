<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useFinanceStore } from '../../stores/finance'

const router = useRouter()
const financeStore = useFinanceStore()

const accounts = computed(() => financeStore.accounts)

function handleRowClick(row: any) {
  router.push(`/finances/accounts/${row.id}`)
}

function goToNewAccount() {
  // TODO: Create new account form
  router.push('/finances/accounts/new')
}
</script>

<template>
  <div class="account-list">
    <h1>Bank Accounts</h1>

    <div class="actions">
      <el-button type="primary" @click="goToNewAccount">New Account</el-button>
    </div>

    <el-table :data="accounts" @row-click="handleRowClick">
      <el-table-column prop="name" label="Name" />
      <el-table-column prop="iban" label="IBAN" />
      <el-table-column prop="bank" label="Bank" />
      <el-table-column prop="balance" label="Balance">
        <template #default="{ row }">
          {{ row.balance.toFixed(2) }} EUR
        </template>
      </el-table-column>
      <el-table-column prop="status" label="Status" />
    </el-table>
  </div>
</template>

<style scoped>
.account-list {
  padding: 20px;
}

.actions {
  margin: 20px 0;
}
</style>
