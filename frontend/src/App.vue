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
  <el-menu v-if="auth.auth" mode="horizontal" :ellipsis="false">
    <el-menu-item index="members">
      <router-link to="/members">Members</router-link>
    </el-menu-item>
    <el-menu-item index="fields">
      <router-link to="/settings/fields">Fields</router-link>
    </el-menu-item>
    <el-menu-item v-if="isSuperAdmin" index="admins">
      <router-link to="/settings/admins">Admins</router-link>
    </el-menu-item>
    <div style="flex-grow: 1" />
    <el-menu-item index="logout" @click="logout">Logout</el-menu-item>
  </el-menu>
  <router-view />
</template>
