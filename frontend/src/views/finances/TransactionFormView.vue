<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const accountId = computed(() => route.params.id as string)
const transactionId = computed(() => route.params.transactionId as string | undefined)

const isEditing = computed(() => !!transactionId.value)
</script>

<template>
  <div class="transaction-form">
    <h1>{{ isEditing ? 'Edit' : 'New' }} Transaction</h1>

    <!-- TODO: Form component will be created in Task 14 -->
    <el-card>
      <p>Transaction form for account {{ accountId }}</p>
      <p v-if="isEditing">Editing transaction: {{ transactionId }}</p>
    </el-card>
  </div>
</template>

<style scoped>
.transaction-form {
  padding: 20px;
}
</style>
