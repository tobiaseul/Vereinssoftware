<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { createMember } from '../api/members'
import MemberForm from '../components/MemberForm.vue'
import type { Member } from '../types'

const router = useRouter()
const qc = useQueryClient()
const draft = ref<Partial<Member>>({})

const { mutate, isPending } = useMutation({
  mutationFn: () => createMember(draft.value as Member),
  onSuccess: (m: Member) => {
    qc.invalidateQueries({ queryKey: ['members'] })
    router.push(`/members/${m.id}`)
  },
})
</script>

<template>
  <div style="padding:24px;max-width:800px;margin:0 auto">
    <h1 style="margin-bottom:24px">New Member</h1>
    <MemberForm v-model="draft" />
    <div style="display:flex;gap:12px;margin-top:24px">
      <el-button type="primary" :loading="isPending" @click="mutate()">
        {{ isPending ? 'Creating...' : 'Create Member' }}
      </el-button>
      <el-button @click="router.push('/members')">Cancel</el-button>
    </div>
  </div>
</template>
