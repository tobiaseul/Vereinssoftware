<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'

const auth = useAuthStore()
const router = useRouter()

const isSuperAdmin = computed(() => auth.auth?.role === 'SuperAdmin')

async function logout() {
  await auth.logout()
  router.push('/login')
}
</script>

<template>
  <el-menu v-if="auth.auth" mode="horizontal" :ellipsis="false" router>
    <el-menu-item index="/members">Members</el-menu-item>
    <el-menu-item index="/settings/fields">Fields</el-menu-item>
    <el-menu-item v-if="isSuperAdmin" index="/settings/admins">Admins</el-menu-item>
    <div style="flex-grow: 1" />
    <el-menu-item index="logout" @click.prevent="logout">Logout</el-menu-item>
  </el-menu>
  <router-view />
</template>
