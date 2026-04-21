import { client } from './client'
import type { AxiosError } from 'axios'

// Interfaces
export interface BankAccount {
  id: string
  name: string
  iban: string
  bank_name: string
  balance: number  // f64 from backend, serialized as number
  is_active: boolean
}

export interface Transaction {
  id: string
  bank_account_id: string
  type: string
  amount: number
  date: string
  member_id?: string
  category: string
  reference: string
  description?: string
  receipt_reference?: string
  reconciled: boolean
  version: number
}

export interface Reconciliation {
  id: string
  bank_account_id: string
  statement_date: string
  status: string
  matched_count: number
}

export interface TransactionFilters {
  category?: string
  type?: string
  reconciled?: boolean
}

export interface StatementLine {
  date: string
  amount: number
  reference: string
}

// Request/Response interfaces
interface CreateAccountRequest {
  name: string
  iban: string
  bank_name: string
}

interface UpdateAccountRequest {
  name: string
  bank_name: string
  is_active: boolean
}

interface CreateTransactionRequest {
  type: string
  amount: string
  date: string
  category: string
  reference: string
  member_id?: string | null
  description?: string
}

interface UpdateTransactionRequest {
  version: number
  amount: string
  date: string
  category: string
  reference: string
  description?: string
}

interface StartReconciliationRequest {
  statement_date: string
  file_name: string
  statement_lines: StatementLine[]
}

interface ConfirmReconciliationRequest {
  matched_transaction_ids: string[]
}

export interface TransactionCategory {
  id: string
  name: string
  created_at: string
}

// Finance API client
export const financeApi = {
  // Bank Accounts
  listAccounts: () =>
    client.get<BankAccount[]>('/api/v1/finance/accounts').then(r => r.data),

  createAccount: (name: string, iban: string, bank_name: string) =>
    client.post<BankAccount>('/api/v1/finance/accounts', {
      name,
      iban,
      bank_name,
    } as CreateAccountRequest).then(r => r.data),

  getAccount: (id: string) =>
    client.get<BankAccount>(`/api/v1/finance/accounts/${id}`).then(r => r.data),

  updateAccount: (id: string, name: string, bank_name: string, is_active: boolean) =>
    client.put<BankAccount>(`/api/v1/finance/accounts/${id}`, {
      name,
      bank_name,
      is_active,
    } as UpdateAccountRequest).then(r => r.data),

  softDeleteAccount: (id: string) =>
    client.delete(`/api/v1/finance/accounts/${id}`).then(r => r.data),

  hardDeleteAccount: (id: string) =>
    client.delete(`/api/v1/finance/accounts/${id}`, { params: { hard: true } }).then(r => r.data),

  // Transactions
  listTransactions: (accountId: string, limit: number, offset: number, filters?: TransactionFilters) => {
    const params = new URLSearchParams()
    params.append('limit', limit.toString())
    params.append('offset', offset.toString())
    if (filters?.category) params.append('category', filters.category)
    if (filters?.type) params.append('type', filters.type)
    if (filters?.reconciled !== undefined) params.append('reconciled', filters.reconciled.toString())
    return client.get<Transaction[]>(`/api/v1/finance/accounts/${accountId}/transactions`, { params }).then(r => r.data)
  },

  listAllTransactions: (limit: number, offset: number, filters?: Record<string, any>) => {
    const params = new URLSearchParams()
    params.append('limit', limit.toString())
    params.append('offset', offset.toString())
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          params.append(key, String(value))
        }
      })
    }
    return client.get<Transaction[]>('/api/v1/finance/transactions', { params }).then(r => r.data)
  },

  createTransaction: (accountId: string, type: string, amount: number, date: string, category: string, reference: string, memberId?: string, description?: string) =>
    client.post<Transaction>(`/api/v1/finance/accounts/${accountId}/transactions`, {
      type,
      amount: amount.toString(),  // Convert to string
      date,
      category,
      reference,
      member_id: memberId || null,  // Convert to null if not provided
      description,
    } as CreateTransactionRequest).then(r => r.data),

  getTransaction: (id: string) =>
    client.get<Transaction>(`/api/v1/finance/transactions/${id}`).then(r => r.data),

  updateTransaction: (id: string, version: number, amount: number, date: string, category: string, reference: string, description?: string) =>
    client.put<Transaction>(`/api/v1/finance/transactions/${id}`, {
      version,
      amount: amount.toString(),  // Convert to string
      date,
      category,
      reference,
      description,
    } as UpdateTransactionRequest).then(r => r.data),

  softDeleteTransaction: (id: string) =>
    client.delete(`/api/v1/finance/transactions/${id}`).then(r => r.data),

  hardDeleteTransaction: (id: string) =>
    client.delete(`/api/v1/finance/transactions/${id}`, { params: { hard: true } }).then(r => r.data),

  // Receipts
  uploadReceipt: (transactionId: string, file: File) => {
    const formData = new FormData()
    formData.append('file', file)
    return client.post(`/api/v1/finance/transactions/${transactionId}/receipt`, formData).then(r => r.data)
  },

  downloadReceipt: (transactionId: string, receiptRef: string) =>
    client.get(`/api/v1/finance/transactions/${transactionId}/receipt/${receiptRef}`, { responseType: 'blob' }).then(r => r.data),

  // Reconciliation
  startReconciliation: (accountId: string, statementDate: string, fileName: string, statementLines: StatementLine[]) =>
    client.post<Reconciliation>(`/api/v1/finance/accounts/${accountId}/reconciliation/start`, {
      statement_date: statementDate,
      file_name: fileName,
      statement_lines: statementLines,
    } as StartReconciliationRequest).then(r => r.data),

  confirmReconciliation: (accountId: string, reconciliationId: string, matchedTransactionIds: string[]) =>
    client.post(`/api/v1/finance/accounts/${accountId}/reconciliation/${reconciliationId}/confirm`, {
      matched_transaction_ids: matchedTransactionIds,
    } as ConfirmReconciliationRequest).then(r => r.data),

  // Categories
  listCategories: () =>
    client.get<TransactionCategory[]>('/api/v1/finance/categories').then(r => r.data),

  createCategory: (name: string) =>
    client.post<TransactionCategory>('/api/v1/finance/categories', { name }).then(r => r.data),

  deleteCategory: (id: string) =>
    client.delete(`/api/v1/finance/categories/${id}`).then(r => r.data),
}
