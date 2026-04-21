<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useFinanceStore } from '../../stores/finance'
import { ElMessage } from 'element-plus'

const router = useRouter()
const financeStore = useFinanceStore()

const form = ref({
  name: '',
  iban: '',
  bank_name: '',
})

const loading = ref(false)
const error = ref<string | null>(null)

async function handleSubmit() {
  error.value = null

  if (!form.value.name || !form.value.iban || !form.value.bank_name) {
    error.value = 'Please fill in all fields'
    ElMessage.error(error.value)
    return
  }

  loading.value = true
  try {
    const account = await financeStore.createAccount(
      form.value.name,
      form.value.iban,
      form.value.bank_name
    )
    ElMessage.success('Account created successfully')
    router.push(`/finances/accounts/${account.id}`)
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Failed to create account'
    error.value = message
    ElMessage.error(message)
    console.error('Error creating account:', err)
  } finally {
    loading.value = false
  }
}

function handleCancel() {
  router.push('/finances/accounts')
}
</script>

<template>
  <div class="account-form-view">
    <h1>New Bank Account</h1>

    <el-card>
      <el-alert v-if="error" :title="error" type="error" :closable="true" @close="error = null" style="margin-bottom: 16px" />

      <el-form @submit.prevent="handleSubmit" label-width="120px">
        <el-form-item label="Account Name" required>
          <el-input v-model="form.name" placeholder="e.g., Main Account" />
        </el-form-item>

        <el-form-item label="IBAN" required>
          <el-input v-model="form.iban" placeholder="DE89370400440532013000" />
        </el-form-item>

        <el-form-item label="Bank Name" required>
          <el-input v-model="form.bank_name" placeholder="e.g., Deutsche Bank" />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="handleSubmit" :loading="loading">Create Account</el-button>
          <el-button @click="handleCancel">Cancel</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<style scoped>
.account-form-view {
  padding: 20px;
}
</style>
