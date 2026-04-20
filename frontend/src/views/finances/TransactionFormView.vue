<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useFinanceStore } from '../../stores/finance'
import TransactionForm from '../../components/finances/TransactionForm.vue'

const route = useRoute()
const financeStore = useFinanceStore()

const accountId = computed(() => route.params.id as string)
const transactionId = computed(() => route.params.transactionId as string | undefined)

const isEditing = computed(() => !!transactionId.value)

const transaction = computed(() => {
  if (!transactionId.value) return undefined
  return financeStore.transactions.find(t => t.id === transactionId.value)
})
</script>

<template>
  <div class="transaction-form-view">
    <h1>{{ isEditing ? 'Edit' : 'New' }} Transaction</h1>

    <el-card>
      <TransactionForm
        :account-id="accountId"
        :transaction="transaction"
        :is-edit-mode="isEditing"
      />
    </el-card>
  </div>
</template>

<style scoped>
.transaction-form-view {
  padding: 20px;
}
</style>
