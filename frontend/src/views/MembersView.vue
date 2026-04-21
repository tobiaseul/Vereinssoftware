<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useQuery } from '@tanstack/vue-query'
import { AgGridVue } from 'ag-grid-vue3'
import type { ColDef, RowClickedEvent } from 'ag-grid-community'
import { getMembers, exportMembers } from '../api/members'
import type { Member } from '../types'

import 'ag-grid-community/styles/ag-grid.css'
import 'ag-grid-community/styles/ag-theme-alpine.css'

const router = useRouter()
const search = ref('')
const membershipType = ref('')
const includeLeft = ref(false)

const { data: members, isLoading } = useQuery({
  queryKey: computed(() => ['members', search.value, membershipType.value, includeLeft.value]),
  queryFn: () => getMembers({
    search: search.value || undefined,
    membership_type: membershipType.value || undefined,
    include_left: includeLeft.value,
  }),
})

const columnDefs: ColDef<Member>[] = [
  {
    headerName: 'Name',
    valueGetter: (p) => `${p.data?.last_name}, ${p.data?.first_name}`,
    flex: 2,
  },
  { field: 'membership_type', headerName: 'Type', flex: 1 },
  { field: 'joined_at', headerName: 'Joined', flex: 1 },
  {
    headerName: 'Status',
    valueGetter: (p) => p.data?.left_at ? 'Left' : 'Active',
    flex: 1,
  },
]

const defaultColDef: ColDef = { sortable: true, filter: true }

function onRowClicked(e: RowClickedEvent<Member>) {
  if (e.data) router.push(`/members/${e.data.id}`)
}

function doExport() {
  exportMembers({
    search: search.value || undefined,
    membership_type: membershipType.value || undefined,
  })
}
</script>

<template>
  <div style="padding:24px;max-width:1100px;margin:0 auto">
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:16px">
      <h1>Members ({{ members?.length ?? 0 }})</h1>
      <div style="display:flex;gap:8px">
        <el-button @click="doExport">Export CSV</el-button>
        <el-button type="primary" @click="router.push('/members/new')">+ New Member</el-button>
      </div>
    </div>

    <div style="display:flex;gap:12px;margin-bottom:16px">
      <el-input v-model="search" placeholder="Search name..." style="flex:1" clearable />
      <el-select v-model="membershipType" placeholder="All types" clearable style="width:180px">
        <el-option label="Aktiv" value="Aktiv" />
        <el-option label="Passiv" value="Passiv" />
        <el-option label="Ehrenmitglied" value="Ehrenmitglied" />
      </el-select>
      <el-checkbox v-model="includeLeft">Include former</el-checkbox>
    </div>

    <div v-if="isLoading">Loading...</div>
    <ag-grid-vue
      v-else
      class="ag-theme-alpine"
      style="width:100%"
      dom-layout="autoHeight"
      :row-data="members ?? []"
      :column-defs="columnDefs"
      :default-col-def="defaultColDef"
      theme="legacy"
      :row-selection="{ mode: 'singleRow' }"
      @row-clicked="onRowClicked"
    />
  </div>
</template>
