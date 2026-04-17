<script setup lang="ts">
import { usePresence } from '../composables/usePresence'

const props = defineProps<{ memberId: string }>()
const { viewers } = usePresence(props.memberId)
</script>

<template>
  <div v-if="viewers.length > 0" style="display:flex;gap:4px;align-items:center">
    <el-tag v-for="viewer in viewers" :key="viewer" size="small" type="info">
      {{ viewer }}
    </el-tag>
  </div>
</template>
