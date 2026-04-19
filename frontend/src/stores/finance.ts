import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { financeApi, type BankAccount, type Transaction } from '../api/finance'

export const useFinanceStore = defineStore('finance', () => {
  // State
  const accounts = ref<BankAccount[]>([])
  const transactions = ref<Transaction[]>([])
  const selectedAccountId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const selectedAccount = computed(() =>
    accounts.value.find(a => a.id === selectedAccountId.value)
  )

  const totalBalance = computed(() =>
    accounts.value.reduce((sum, account) => sum + account.balance, 0)
  )

  // Actions
  async function loadAccounts() {
    isLoading.value = true
    error.value = null
    try {
      const data = await financeApi.listAccounts()
      accounts.value = data
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load accounts'
    } finally {
      isLoading.value = false
    }
  }

  async function createAccount(
    name: string,
    iban: string,
    bank_name: string
  ): Promise<BankAccount> {
    try {
      const account = await financeApi.createAccount(name, iban, bank_name)
      accounts.value.push(account)
      ElMessage.success('Account created successfully')
      return account
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create account'
      throw err
    }
  }

  async function loadTransactions(
    accountId: string,
    limit: number = 50,
    offset: number = 0
  ) {
    selectedAccountId.value = accountId
    isLoading.value = true
    error.value = null
    try {
      const data = await financeApi.listTransactions(accountId, limit, offset)
      transactions.value = data
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load transactions'
    } finally {
      isLoading.value = false
    }
  }

  async function createTransaction(
    accountId: string,
    type: string,
    amount: number,
    date: string,
    category: string,
    reference: string,
    memberId?: string,
    description?: string
  ): Promise<Transaction> {
    try {
      const transaction = await financeApi.createTransaction(
        accountId,
        type,
        amount,
        date,
        category,
        reference,
        memberId,
        description
      )
      if (selectedAccountId.value === accountId) {
        transactions.value.unshift(transaction)
      }
      ElMessage.success('Transaction created successfully')
      return transaction
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create transaction'
      throw err
    }
  }

  async function uploadReceipt(transactionId: string, file: File): Promise<void> {
    try {
      await financeApi.uploadReceipt(transactionId, file)
      ElMessage.success('Receipt uploaded successfully')
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to upload receipt'
      throw err
    }
  }

  return {
    // State
    accounts,
    transactions,
    selectedAccountId,
    isLoading,
    error,
    // Computed
    selectedAccount,
    totalBalance,
    // Actions
    loadAccounts,
    createAccount,
    loadTransactions,
    createTransaction,
    uploadReceipt,
  }
})
