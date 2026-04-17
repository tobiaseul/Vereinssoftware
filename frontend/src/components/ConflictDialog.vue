<script setup lang="ts">
import type { Member } from '../types'

const props = defineProps<{
  myDraft: Partial<Member>
  serverMember: Member
  onResolve: (merged: Member) => void
  onDiscard: () => void
}>()

const FIELDS: (keyof Member)[] = ['first_name', 'last_name', 'email', 'phone', 'membership_type', 'notes']

const diffFields = FIELDS.filter(
  (k) => props.myDraft[k] !== props.serverMember[k]
)

function keepMine() {
  props.onResolve({ ...props.serverMember, ...props.myDraft, version: props.serverMember.version })
}
</script>

<template>
  <el-dialog title="Save Conflict" :model-value="true" :close-on-click-modal="false" :show-close="false" width="600px">
    <p style="margin-bottom:12px">Someone else saved changes while you were editing. Review the differences:</p>
    <el-table :data="diffFields.map(k => ({ field: k, mine: myDraft[k], server: serverMember[k] }))">
      <el-table-column prop="field" label="Field" width="150" />
      <el-table-column prop="mine" label="Your version" />
      <el-table-column prop="server" label="Server version" />
    </el-table>
    <template #footer>
      <el-button type="primary" @click="keepMine">Keep my changes</el-button>
      <el-button @click="onDiscard">Use server version</el-button>
    </template>
  </el-dialog>
</template>
