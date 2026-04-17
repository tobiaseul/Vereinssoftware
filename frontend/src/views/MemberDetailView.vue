<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { AxiosError } from 'axios'
import { getMember, updateMember, deleteMember } from '../api/members'
import MemberForm from '../components/MemberForm.vue'
import ConflictDialog from '../components/ConflictDialog.vue'
import PresenceIndicator from '../components/PresenceIndicator.vue'
import type { Member } from '../types'

const route = useRoute()
const router = useRouter()
const qc = useQueryClient()

const id = route.params.id as string
const draft = ref<Partial<Member> | null>(null)
const conflict = ref<{ serverMember: Member; myDraft: Partial<Member> } | null>(null)

const { data: member, isLoading } = useQuery({
  queryKey: ['member', id],
  queryFn: () => getMember(id),
})

watch(member, (m) => { if (m && !draft.value) draft.value = { ...m } })

const { mutate: save, isPending: isSaving } = useMutation({
  mutationFn: (data: Partial<Member>) => updateMember(id, data as Member),
  onSuccess: (updated: Member) => {
    qc.setQueryData(['member', id], updated)
    draft.value = { ...updated }
  },
  onError: (err: unknown) => {
    if (err instanceof AxiosError && err.response?.status === 409) {
      getMember(id).then((serverMember) => {
        conflict.value = { serverMember, myDraft: draft.value! }
      })
    }
  },
})

const { mutate: remove } = useMutation({
  mutationFn: () => deleteMember(id),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['members'] })
    router.push('/members')
  },
})

function resolveConflict(resolved: Member) {
  conflict.value = null
  draft.value = { ...resolved }
  save(resolved)
}

function confirmRemove() {
  if (window.confirm('Mark as left?')) remove()
}

function discardConflict() {
  if (!conflict.value) return
  draft.value = { ...conflict.value.serverMember }
  conflict.value = null
}
</script>

<template>
  <div style="padding:24px;max-width:800px;margin:0 auto">
    <div v-if="isLoading || !draft">Loading...</div>
    <template v-else>
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:24px">
        <h1>{{ member?.first_name }} {{ member?.last_name }}</h1>
        <PresenceIndicator :member-id="id" />
      </div>

      <MemberForm v-model="draft as Partial<Member>" />

      <div style="display:flex;gap:12px;margin-top:24px">
        <el-button type="primary" :loading="isSaving" @click="save(draft!)">
          {{ isSaving ? 'Saving...' : 'Save' }}
        </el-button>
        <el-button @click="router.push('/members')">Cancel</el-button>
        <el-button type="danger" style="margin-left:auto"
          @click="confirmRemove">
          Mark as Left
        </el-button>
      </div>

      <ConflictDialog
        v-if="conflict"
        :my-draft="conflict.myDraft"
        :server-member="conflict.serverMember"
        :on-resolve="resolveConflict"
        :on-discard="discardConflict"
      />
    </template>
  </div>
</template>
