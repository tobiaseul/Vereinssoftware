<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const router = useRouter()

const username = ref('')
const password = ref('')
const error = ref<string | null>(null)
const loading = ref(false)

async function submit() {
  error.value = null
  loading.value = true
  try {
    await auth.login(username.value, password.value)
    router.push('/members')
  } catch {
    error.value = 'Invalid username or password'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div style="display:flex;justify-content:center;padding-top:80px">
    <el-card style="width:360px">
      <template #header><b>Login</b></template>
      <el-form @submit.prevent="submit" label-position="top">
        <el-form-item label="Username">
          <el-input v-model="username" autocomplete="username" />
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="password" type="password" autocomplete="current-password" />
        </el-form-item>
        <el-alert v-if="error" :title="error" type="error" :closable="false" style="margin-bottom:12px" />
        <el-button type="primary" native-type="submit" :loading="loading" style="width:100%">
          Login
        </el-button>
      </el-form>
    </el-card>
  </div>
</template>
