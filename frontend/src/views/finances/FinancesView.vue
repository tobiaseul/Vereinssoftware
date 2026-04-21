<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useFinanceStore } from '../../stores/finance'

const router = useRouter()
const financeStore = useFinanceStore()

onMounted(() => {
  financeStore.loadAccounts()
})

const accounts = computed(() => financeStore.accounts)

function handleRowClick(row: any) {
  router.push(`/finances/accounts/${row.id}`)
}

function goToNewAccount() {
  router.push('/finances/accounts/new')
}
</script>

<template>
  <div class="bank-accounts">
    <h1>Bank Accounts</h1>

    <div class="actions">
      <el-button type="primary" @click="goToNewAccount">New Account</el-button>
    </div>

    <el-table :data="accounts" @row-click="handleRowClick">
      <el-table-column prop="name" label="Name" />
      <el-table-column prop="iban" label="IBAN" />
      <el-table-column prop="bank_name" label="Bank" />
      <el-table-column prop="balance" label="Balance">
        <template #default="{ row }">
          {{ row.balance.toFixed(2) }} EUR
        </template>
      </el-table-column>
      <el-table-column prop="is_active" label="Status">
        <template #default="{ row }">
          <el-tag :type="row.is_active ? 'success' : 'info'">
            {{ row.is_active ? 'Active' : 'Inactive' }}
          </el-tag>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<style scoped>
.bank-accounts {
  padding: 20px;
}

.actions {
  margin: 20px 0;
}
</style>
