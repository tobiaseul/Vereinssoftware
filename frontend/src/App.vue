<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'

const auth = useAuthStore()
const router = useRouter()

const isSuperAdmin = computed(() => auth.auth?.role === 'SuperAdmin')
const hasFinanceRole = computed(() => auth.hasFinanceRole)

async function logout() {
  await auth.logout()
  router.push('/login')
}
</script>

<template>
  <el-menu v-if="auth.auth" mode="horizontal" :ellipsis="false" router>
    <el-menu-item index="/members">Members</el-menu-item>
    <el-menu-item v-if="hasFinanceRole" index="/finances">Bank Accounts</el-menu-item>
    <el-menu-item v-if="hasFinanceRole" index="/finances/transactions">Transactions</el-menu-item>
    <el-sub-menu v-if="isSuperAdmin" index="/settings">
      <template #title>Settings</template>
      <el-menu-item index="/settings/admins">Admins</el-menu-item>
      <el-menu-item index="/settings/configuration/fields">Configuration</el-menu-item>
    </el-sub-menu>
    <div style="flex-grow: 1" />
    <el-menu-item index="logout" @click.prevent="logout">Logout</el-menu-item>
  </el-menu>
  <router-view />
</template>
