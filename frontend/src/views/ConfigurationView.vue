<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const tabs = [
  { label: 'Member Fields', name: 'fields', path: '/settings/configuration/fields' },
  { label: 'Transaction Categories', name: 'categories', path: '/settings/configuration/categories' },
]

const activeTab = computed(() => {
  const lastSegment = route.path.split('/').pop()
  return lastSegment === 'fields' ? 'fields' : 'categories'
})

function handleTabChange(tabName: string) {
  const tab = tabs.find(t => t.name === tabName)
  if (tab) {
    router.push(tab.path)
  }
}
</script>

<template>
  <div class="configuration-view">
    <h1>Configuration</h1>

    <el-tabs :model-value="activeTab" @tab-change="handleTabChange">
      <el-tab-pane
        v-for="tab in tabs"
        :key="tab.name"
        :label="tab.label"
        :name="tab.name"
      />
    </el-tabs>

    <router-view />
  </div>
</template>

<style scoped>
.configuration-view {
  padding: 20px;
}

h1 {
  margin-bottom: 20px;
}
</style>
