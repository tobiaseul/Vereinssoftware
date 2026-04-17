# Vue 3 Frontend Rewrite — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the React+Tailwind frontend with a Vue 3 + Element Plus + AG Grid frontend at feature parity.

**Architecture:** Delete `frontend/` entirely and scaffold a fresh Vue 3+Vite+TypeScript project in its place. Carry over `src/api/`, `src/types/index.ts`, and the Vite proxy config unchanged. All React components are rewritten as Vue SFCs using Element Plus, with AG Grid for the members list and Pinia for auth state.

**Tech Stack:** Vue 3 (Composition API, `<script setup>`), TypeScript, Vite, Vue Router 4, @tanstack/vue-query, Axios, Pinia, Element Plus, AG Grid Community + ag-grid-vue3, Vitest + @vue/test-utils

---

## File Map

| File | Action | Purpose |
|---|---|---|
| `frontend/` (all React files) | Delete | Remove entire React project |
| `frontend/package.json` | Create | Vue 3 dependencies |
| `frontend/vite.config.ts` | Create | Vue plugin + proxy (same proxy as React) |
| `frontend/src/main.ts` | Create | App bootstrap: Vue app + plugins |
| `frontend/src/App.vue` | Create | ElMenu nav bar, `<RouterView>` |
| `frontend/src/api/` | Carry over | No changes — plain TypeScript |
| `frontend/src/types/index.ts` | Carry over | No changes |
| `frontend/src/stores/auth.ts` | Create | Pinia store: auth state, login, logout, silentRefresh |
| `frontend/src/router/index.ts` | Create | Vue Router 4 + navigation guards |
| `frontend/src/composables/usePresence.ts` | Create | WebSocket presence composable |
| `frontend/src/views/LoginView.vue` | Create | Login form |
| `frontend/src/views/MembersView.vue` | Create | AG Grid members table |
| `frontend/src/views/MemberDetailView.vue` | Create | Edit form + conflict + presence |
| `frontend/src/views/MemberNewView.vue` | Create | Create member form |
| `frontend/src/views/FieldsView.vue` | Create | Field list with inline edit + enum options |
| `frontend/src/views/AdminsView.vue` | Create | Admin CRUD |
| `frontend/src/components/MemberForm.vue` | Create | Shared form used by Detail + New |
| `frontend/src/components/ConflictDialog.vue` | Create | ElDialog diff view |
| `frontend/src/components/PresenceIndicator.vue` | Create | Badge of concurrent viewers |
| `frontend/src/tests/LoginView.test.ts` | Create | Login form submits credentials |
| `frontend/src/tests/ConflictDialog.test.ts` | Create | Conflict dialog renders and calls handlers |

---

### Task 1: Scaffold Vue 3 project

**Files:**
- Delete: `frontend/` (entire directory)
- Create: `frontend/package.json`
- Create: `frontend/tsconfig.json`
- Create: `frontend/vite.config.ts`
- Carry over: `frontend/src/api/` and `frontend/src/types/index.ts`

- [ ] **Step 1: Delete the React project and scaffold fresh**

```bash
cd /Users/tobi/Documents/Coding/Vereinssoftware
rm -rf frontend
npm create vite@latest frontend -- --template vue-ts
cd frontend
```

- [ ] **Step 2: Install all dependencies**

```bash
npm install \
  vue-router@4 \
  pinia \
  @tanstack/vue-query \
  axios \
  element-plus \
  @element-plus/icons-vue \
  ag-grid-community \
  ag-grid-vue3

npm install -D \
  vitest \
  @vue/test-utils \
  jsdom \
  @vitejs/plugin-vue \
  unplugin-vue-components \
  unplugin-auto-import
```

- [ ] **Step 3: Restore api/ and types/ from git**

The api/ and types/ files are plain TypeScript — carry them over from the React project. Since the React project was just deleted, restore from git:

```bash
git checkout HEAD -- frontend/src/api frontend/src/types
```

Expected: `frontend/src/api/client.ts`, `frontend/src/api/auth.ts`, `frontend/src/api/members.ts`, `frontend/src/api/fieldDefinitions.ts`, `frontend/src/api/admins.ts`, and `frontend/src/types/index.ts` are restored.

- [ ] **Step 4: Replace vite.config.ts**

Replace the scaffolded `frontend/vite.config.ts` entirely with:

```ts
import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    proxy: {
      '/api': 'http://localhost:3000',
      '/auth': 'http://localhost:3000',
      '/ws': { target: 'ws://localhost:3000', ws: true },
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
  },
})
```

- [ ] **Step 5: Update tsconfig.json**

Replace `frontend/tsconfig.json` with:

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "jsx": "preserve",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "lib": ["ESNext", "DOM"],
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.vue"]
}
```

- [ ] **Step 6: Verify TypeScript compiles**

```bash
cd frontend && npx tsc --noEmit
```

Expected: no errors (api/ and types/ files compile cleanly).

- [ ] **Step 7: Commit**

```bash
git checkout -b vue3-frontend
git add frontend/
git commit -m "feat: scaffold Vue 3 project, carry over api + types"
```

---

### Task 2: Pinia auth store + Vue Router

**Files:**
- Create: `frontend/src/stores/auth.ts`
- Create: `frontend/src/router/index.ts`

- [ ] **Step 1: Create the Pinia auth store**

Create `frontend/src/stores/auth.ts`:

```ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { login as apiLogin, logout as apiLogout } from '../api/auth'
import { setAccessToken } from '../api/client'
import type { AuthState } from '../types'

export const useAuthStore = defineStore('auth', () => {
  const auth = ref<AuthState | null>(null)
  const isLoading = ref(true)

  async function login(username: string, password: string) {
    const result = await apiLogin(username, password)
    auth.value = result
    setAccessToken(result.access_token)
  }

  async function logout() {
    await apiLogout()
    auth.value = null
    setAccessToken(null)
  }

  async function silentRefresh() {
    try {
      const result = await apiLogin('', '')
      // The client interceptor handles the actual refresh call via /auth/refresh
      // This call just populates auth from the stored session
      auth.value = result
      setAccessToken(result.access_token)
    } catch {
      auth.value = null
    } finally {
      isLoading.value = false
    }
  }

  return { auth, isLoading, login, logout, silentRefresh }
})
```

**Note:** `silentRefresh` calls `GET /auth/me` (or the interceptor-driven refresh flow). The actual silent refresh on app mount works by calling `client.get('/auth/refresh')` to get a new token. Update the body to:

```ts
  async function silentRefresh() {
    try {
      const { data } = await import('../api/client').then(m => m.client.post<AuthState>('/auth/refresh'))
      auth.value = data
      setAccessToken(data.access_token)
    } catch {
      auth.value = null
    } finally {
      isLoading.value = false
    }
  }
```

- [ ] **Step 2: Create the router**

Create `frontend/src/router/index.ts`:

```ts
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/login', component: () => import('../views/LoginView.vue') },
    {
      path: '/members',
      component: () => import('../views/MembersView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/members/new',
      component: () => import('../views/MemberNewView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/members/:id',
      component: () => import('../views/MemberDetailView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/settings/fields',
      component: () => import('../views/FieldsView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/settings/admins',
      component: () => import('../views/AdminsView.vue'),
      meta: { requiresAuth: true, requiresSuperAdmin: true },
    },
    { path: '/:pathMatch(.*)*', redirect: '/members' },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (auth.isLoading) await auth.silentRefresh()
  if (to.meta.requiresAuth && !auth.auth) return '/login'
  if (to.meta.requiresSuperAdmin && auth.auth?.role !== 'SuperAdmin') return '/members'
})

export default router
```

- [ ] **Step 3: Verify TypeScript compiles**

```bash
cd frontend && npx tsc --noEmit
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/stores frontend/src/router
git commit -m "feat: add Pinia auth store and Vue Router with guards"
```

---

### Task 3: main.ts + App.vue + LoginView.vue

**Files:**
- Modify: `frontend/src/main.ts`
- Create: `frontend/src/App.vue`
- Create: `frontend/src/views/LoginView.vue`

- [ ] **Step 1: Write main.ts**

Replace the scaffolded `frontend/src/main.ts`:

```ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import { VueQueryPlugin } from '@tanstack/vue-query'
import router from './router'
import App from './App.vue'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(ElementPlus)
app.use(VueQueryPlugin)
app.mount('#app')
```

- [ ] **Step 2: Write App.vue**

Create `frontend/src/App.vue`:

```vue
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
```

- [ ] **Step 3: Write LoginView.vue**

Create `frontend/src/views/LoginView.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const router = useRouter()

const username = ref('')
const password = ref('')
const error = ref<string | null>(null)
const loading = ref(false)

async function submit() {
  error.value = null
  loading.value = true
  try {
    await auth.login(username.value, password.value)
    router.push('/members')
  } catch {
    error.value = 'Invalid username or password'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div style="display:flex;justify-content:center;padding-top:80px">
    <el-card style="width:360px">
      <template #header><b>Login</b></template>
      <el-form @submit.prevent="submit" label-position="top">
        <el-form-item label="Username">
          <el-input v-model="username" autocomplete="username" />
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="password" type="password" autocomplete="current-password" />
        </el-form-item>
        <el-alert v-if="error" :title="error" type="error" :closable="false" style="margin-bottom:12px" />
        <el-button type="primary" native-type="submit" :loading="loading" style="width:100%">
          Login
        </el-button>
      </el-form>
    </el-card>
  </div>
</template>
```

- [ ] **Step 4: Verify build**

```bash
cd frontend && npm run build
```

Expected: builds without errors (only LoginView route resolves so far — other views are lazy-loaded and not checked at build time).

- [ ] **Step 5: Commit**

```bash
git add frontend/src/main.ts frontend/src/App.vue frontend/src/views/LoginView.vue
git commit -m "feat: add main.ts, App.vue nav, LoginView"
```

---

### Task 4: MembersView.vue (AG Grid)

**Files:**
- Create: `frontend/src/views/MembersView.vue`

The React version used a plain HTML table with server-side filtering via query params. The Vue version uses AG Grid with client-side quick filter; server filtering params are passed through `useQuery`.

- [ ] **Step 1: Write MembersView.vue**

Create `frontend/src/views/MembersView.vue`:

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useQuery } from '@tanstack/vue-query'
import { AgGridVue } from 'ag-grid-vue3'
import type { ColDef, GridReadyEvent, RowClickedEvent } from 'ag-grid-community'
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
      row-selection="single"
      @row-clicked="onRowClicked"
    />
  </div>
</template>
```

- [ ] **Step 2: Verify build**

```bash
cd frontend && npm run build
```

Expected: builds without errors.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/views/MembersView.vue
git commit -m "feat: add MembersView with AG Grid"
```

---

### Task 5: usePresence composable + PresenceIndicator

**Files:**
- Create: `frontend/src/composables/usePresence.ts`
- Create: `frontend/src/components/PresenceIndicator.vue`

- [ ] **Step 1: Write usePresence.ts**

The React hook used `useEffect` + cleanup. The Vue composable uses `onMounted`/`onUnmounted`.

Create `frontend/src/composables/usePresence.ts`:

```ts
import { ref, onMounted, onUnmounted } from 'vue'
import { getAccessToken } from '../api/client'

export function usePresence(memberId: string) {
  const viewers = ref<string[]>([])
  let ws: WebSocket | null = null

  onMounted(() => {
    const token = getAccessToken()
    if (!token) return
    ws = new WebSocket(`ws://localhost:3000/ws?token=${token}`)

    ws.onopen = () => {
      ws!.send(JSON.stringify({ type: 'viewing', member_id: memberId }))
    }

    ws.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data)
        if (msg.viewers) viewers.value = msg.viewers
      } catch {
        // ignore malformed messages
      }
    }
  })

  onUnmounted(() => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'left' }))
      ws.close()
    }
  })

  return { viewers }
}
```

- [ ] **Step 2: Write PresenceIndicator.vue**

Create `frontend/src/components/PresenceIndicator.vue`:

```vue
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
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/composables/usePresence.ts frontend/src/components/PresenceIndicator.vue
git commit -m "feat: add usePresence composable and PresenceIndicator"
```

---

### Task 6: MemberForm.vue

**Files:**
- Create: `frontend/src/components/MemberForm.vue`

The form is shared by MemberDetailView and MemberNewView. Enum custom fields render as `ElSelect`, boolean as `ElCheckbox`, others as `ElInput`.

- [ ] **Step 1: Write MemberForm.vue**

Create `frontend/src/components/MemberForm.vue`:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { getFieldDefinitions } from '../api/fieldDefinitions'
import type { Member, MembershipType } from '../types'

const props = defineProps<{ modelValue: Partial<Member>; disabled?: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: Partial<Member>): void }>()

const { data: fields } = useQuery({
  queryKey: ['field-definitions'],
  queryFn: getFieldDefinitions,
})

const cf = computed(() => (props.modelValue.custom_fields as Record<string, unknown>) ?? {})

function set(key: keyof Member, val: unknown) {
  emit('update:modelValue', { ...props.modelValue, [key]: val })
}

function setCf(name: string, val: unknown) {
  set('custom_fields', { ...cf.value, [name]: val })
}
</script>

<template>
  <el-form label-position="top">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px">
      <el-form-item label="First Name *">
        <el-input :model-value="modelValue.first_name ?? ''"
          @update:model-value="set('first_name', $event)" :disabled="disabled" />
      </el-form-item>
      <el-form-item label="Last Name *">
        <el-input :model-value="modelValue.last_name ?? ''"
          @update:model-value="set('last_name', $event)" :disabled="disabled" />
      </el-form-item>
      <el-form-item label="Email">
        <el-input type="email" :model-value="modelValue.email ?? ''"
          @update:model-value="set('email', $event || null)" :disabled="disabled" />
      </el-form-item>
      <el-form-item label="Phone">
        <el-input :model-value="modelValue.phone ?? ''"
          @update:model-value="set('phone', $event || null)" :disabled="disabled" />
      </el-form-item>
    </div>

    <el-form-item label="Membership Type *">
      <el-select :model-value="modelValue.membership_type ?? ''"
        @update:model-value="set('membership_type', $event as MembershipType)" :disabled="disabled">
        <el-option label="Aktiv" value="Aktiv" />
        <el-option label="Passiv" value="Passiv" />
        <el-option label="Ehrenmitglied" value="Ehrenmitglied" />
      </el-select>
    </el-form-item>

    <el-form-item label="Notes">
      <el-input type="textarea" :rows="3" :model-value="modelValue.notes ?? ''"
        @update:model-value="set('notes', $event || null)" :disabled="disabled" />
    </el-form-item>

    <template v-if="fields && fields.length > 0">
      <el-divider />
      <h3 style="margin-bottom:12px">Custom Fields</h3>
      <el-form-item v-for="f in fields" :key="f.id" :label="f.name + (f.required ? ' *' : '')">
        <el-select v-if="f.field_type === 'enum'"
          :model-value="cf[f.name] as string ?? ''"
          @update:model-value="setCf(f.name, $event || null)"
          :disabled="disabled" clearable>
          <el-option v-for="opt in f.options" :key="opt.id" :label="opt.value" :value="opt.value" />
        </el-select>
        <el-checkbox v-else-if="f.field_type === 'boolean'"
          :model-value="!!cf[f.name]"
          @update:model-value="setCf(f.name, $event)"
          :disabled="disabled" />
        <el-input v-else
          :type="f.field_type === 'number' ? 'number' : f.field_type === 'date' ? 'date' : 'text'"
          :model-value="cf[f.name] as string ?? ''"
          @update:model-value="setCf(f.name, $event || null)"
          :disabled="disabled" />
      </el-form-item>
    </template>
  </el-form>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/components/MemberForm.vue
git commit -m "feat: add shared MemberForm component"
```

---

### Task 7: ConflictDialog.vue + MemberDetailView.vue

**Files:**
- Create: `frontend/src/components/ConflictDialog.vue`
- Create: `frontend/src/views/MemberDetailView.vue`

- [ ] **Step 1: Write ConflictDialog.vue**

The dialog shows only the fields that differ between `myDraft` and `serverMember`. Fields checked: `first_name`, `last_name`, `email`, `phone`, `membership_type`, `notes`.

Create `frontend/src/components/ConflictDialog.vue`:

```vue
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
```

- [ ] **Step 2: Write MemberDetailView.vue**

Create `frontend/src/views/MemberDetailView.vue`:

```vue
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
  onError: async (err: unknown) => {
    if (err instanceof AxiosError && err.response?.status === 409) {
      const serverMember = await getMember(id)
      conflict.value = { serverMember, myDraft: draft.value! }
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

      <MemberForm v-model="draft" />

      <div style="display:flex;gap:12px;margin-top:24px">
        <el-button type="primary" :loading="isSaving" @click="save(draft!)">
          {{ isSaving ? 'Saving...' : 'Save' }}
        </el-button>
        <el-button @click="router.push('/members')">Cancel</el-button>
        <el-button type="danger" style="margin-left:auto"
          @click="() => { if (confirm('Mark as left?')) remove() }">
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
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/ConflictDialog.vue frontend/src/views/MemberDetailView.vue
git commit -m "feat: add ConflictDialog and MemberDetailView"
```

---

### Task 8: MemberNewView.vue + AdminsView.vue

**Files:**
- Create: `frontend/src/views/MemberNewView.vue`
- Create: `frontend/src/views/AdminsView.vue`

- [ ] **Step 1: Write MemberNewView.vue**

Create `frontend/src/views/MemberNewView.vue`:

```vue
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
```

- [ ] **Step 2: Write AdminsView.vue**

Create `frontend/src/views/AdminsView.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { getAdmins, createAdmin, deleteAdmin } from '../api/admins'
import { useAuthStore } from '../stores/auth'
import type { AdminRole } from '../types'

const auth = useAuthStore()
const qc = useQueryClient()

const form = ref({ username: '', password: '', role: 'Admin' as AdminRole })
const error = ref<string | null>(null)

const { data: admins } = useQuery({ queryKey: ['admins'], queryFn: getAdmins })

const add = useMutation({
  mutationFn: () => createAdmin(form.value),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['admins'] })
    form.value = { username: '', password: '', role: 'Admin' }
    error.value = null
  },
  onError: (err: unknown) => {
    error.value = err instanceof Error ? err.message : 'Failed to add'
  },
})

const remove = useMutation({
  mutationFn: (id: string) => deleteAdmin(id),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['admins'] })
    error.value = null
  },
  onError: (err: unknown) => {
    error.value = err instanceof Error ? err.message : 'Failed to delete'
  },
})
</script>

<template>
  <div style="padding:24px;max-width:600px;margin:0 auto">
    <h1 style="margin-bottom:16px">Admin Users</h1>
    <el-alert v-if="error" :title="error" type="error" :closable="false" style="margin-bottom:12px" />

    <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-bottom:16px">
      <el-input v-model="form.username" placeholder="Username" />
      <el-input v-model="form.password" type="password" placeholder="Password" />
      <el-select v-model="form.role">
        <el-option label="Admin" value="Admin" />
        <el-option label="SuperAdmin" value="SuperAdmin" />
      </el-select>
      <el-button type="primary" :disabled="!form.username || !form.password || add.isPending.value"
        @click="add.mutate()">
        Add Admin
      </el-button>
    </div>

    <el-table :data="admins ?? []" style="width:100%">
      <el-table-column label="Username">
        <template #default="{ row }">
          {{ row.username }}
          <el-tag v-if="row.id === auth.auth?.admin_id" size="small" style="margin-left:6px">you</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="role" label="Role" />
      <el-table-column label="" width="100">
        <template #default="{ row }">
          <el-button v-if="row.id !== auth.auth?.admin_id" type="danger" size="small" text
            :loading="remove.isPending.value"
            @click="() => { if (confirm(`Remove ${row.username}?`)) remove.mutate(row.id) }">
            Remove
          </el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/views/MemberNewView.vue frontend/src/views/AdminsView.vue
git commit -m "feat: add MemberNewView and AdminsView"
```

---

### Task 9: FieldsView.vue

**Files:**
- Create: `frontend/src/views/FieldsView.vue`

Fields list with inline edit row and expandable enum options section per field.

- [ ] **Step 1: Write FieldsView.vue**

Create `frontend/src/views/FieldsView.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  getFieldDefinitions, createFieldDefinition, updateFieldDefinition, deleteFieldDefinition,
  addFieldOption, updateFieldOption, deleteFieldOption,
} from '../api/fieldDefinitions'
import type { FieldDefinition, FieldType } from '../types'

const qc = useQueryClient()
const error = ref<string | null>(null)

// Add form
const form = ref({ name: '', field_type: 'text' as FieldType, required: false })

// Edit state
const editingId = ref<string | null>(null)
const editForm = ref({ name: '', required: false })

// Expanded enum options
const expandedId = ref<string | null>(null)

// Option edit state
const editingOptionId = ref<string | null>(null)
const editOptionValue = ref('')
const newOptionValue = ref('')

const { data: fields } = useQuery({ queryKey: ['field-definitions'], queryFn: getFieldDefinitions })

const addField = useMutation({
  mutationFn: () => createFieldDefinition(form.value),
  onSuccess: (newField) => {
    qc.invalidateQueries({ queryKey: ['field-definitions'] })
    form.value = { name: '', field_type: 'text', required: false }
    error.value = null
    if (newField.field_type === 'enum') expandedId.value = newField.id
  },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to add field' },
})

const updateField = useMutation({
  mutationFn: ({ id, ...data }: { id: string; name: string; required: boolean }) =>
    updateFieldDefinition(id, data),
  onSuccess: () => {
    qc.invalidateQueries({ queryKey: ['field-definitions'] })
    editingId.value = null
    error.value = null
  },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to update field' },
})

const removeField = useMutation({
  mutationFn: (id: string) => deleteFieldDefinition(id),
  onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); error.value = null },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to delete field' },
})

function startEdit(f: FieldDefinition) {
  editingId.value = f.id
  editForm.value = { name: f.name, required: f.required }
}

const addOption = useMutation({
  mutationFn: ({ fieldId, value }: { fieldId: string; value: string }) =>
    addFieldOption(fieldId, { value }),
  onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); newOptionValue.value = ''; error.value = null },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to add option' },
})

const updateOption = useMutation({
  mutationFn: ({ fieldId, optionId, value }: { fieldId: string; optionId: string; value: string }) =>
    updateFieldOption(fieldId, optionId, { value }),
  onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); editingOptionId.value = null; error.value = null },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to update option' },
})

const removeOption = useMutation({
  mutationFn: ({ fieldId, optionId }: { fieldId: string; optionId: string }) =>
    deleteFieldOption(fieldId, optionId),
  onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); error.value = null },
  onError: (err: unknown) => { error.value = err instanceof Error ? err.message : 'Failed to delete option' },
})
</script>

<template>
  <div style="padding:24px;max-width:640px;margin:0 auto">
    <h1 style="margin-bottom:16px">Custom Fields</h1>
    <el-alert v-if="error" :title="error" type="error" :closable="false" style="margin-bottom:12px" />

    <!-- Add field form -->
    <div style="display:flex;gap:8px;margin-bottom:16px;align-items:center">
      <el-input v-model="form.name" placeholder="Field name..." style="flex:1" />
      <el-select v-model="form.field_type" style="width:140px">
        <el-option label="Text" value="text" />
        <el-option label="Number" value="number" />
        <el-option label="Date" value="date" />
        <el-option label="Boolean" value="boolean" />
        <el-option label="Dropdown" value="enum" />
      </el-select>
      <el-checkbox v-model="form.required">Required</el-checkbox>
      <el-button type="primary" :disabled="!form.name || addField.isPending.value" @click="addField.mutate()">
        Add
      </el-button>
    </div>

    <!-- Field list -->
    <div v-for="f in fields ?? []" :key="f.id" style="border:1px solid #e4e7ed;border-radius:4px;margin-bottom:8px;padding:12px">
      <!-- Edit row -->
      <template v-if="editingId === f.id">
        <div style="display:flex;gap:8px;align-items:center">
          <el-input v-model="editForm.name" style="flex:1" />
          <el-checkbox v-model="editForm.required">Required</el-checkbox>
          <el-button type="success" size="small"
            :disabled="!editForm.name || updateField.isPending.value"
            @click="updateField.mutate({ id: f.id, ...editForm })">Save</el-button>
          <el-button size="small" @click="editingId = null">Cancel</el-button>
        </div>
      </template>

      <!-- Display row -->
      <template v-else>
        <div style="display:flex;justify-content:space-between;align-items:center">
          <div style="display:flex;align-items:center;gap:8px">
            <el-button v-if="f.field_type === 'enum'" text size="small"
              @click="expandedId = expandedId === f.id ? null : f.id">
              {{ expandedId === f.id ? '▼' : '▶' }}
            </el-button>
            <span>
              {{ f.name }}
              <span style="color:#909399;font-size:13px">
                ({{ f.field_type === 'enum' ? `dropdown, ${f.options.length} options` : f.field_type }}{{ f.required ? ', required' : '' }})
              </span>
            </span>
          </div>
          <div style="display:flex;gap:8px">
            <el-button type="primary" size="small" text @click="startEdit(f)">Edit</el-button>
            <el-button type="danger" size="small" text :loading="removeField.isPending.value"
              @click="() => { if (confirm(`Remove '${f.name}' and all its options?`)) removeField.mutate(f.id) }">
              Remove
            </el-button>
          </div>
        </div>
      </template>

      <!-- Enum options panel -->
      <div v-if="expandedId === f.id && f.field_type === 'enum'"
        style="margin-top:12px;padding-left:16px;border-left:2px solid #e4e7ed">
        <div v-for="opt in f.options" :key="opt.id" style="display:flex;align-items:center;gap:8px;margin-bottom:6px">
          <template v-if="editingOptionId === opt.id">
            <el-input v-model="editOptionValue" size="small" style="flex:1" />
            <el-button size="small" type="success" :disabled="!editOptionValue || updateOption.isPending.value"
              @click="updateOption.mutate({ fieldId: f.id, optionId: opt.id, value: editOptionValue })">Save</el-button>
            <el-button size="small" @click="editingOptionId = null">Cancel</el-button>
          </template>
          <template v-else>
            <span style="flex:1;font-size:14px">{{ opt.value }}</span>
            <el-button size="small" text type="primary"
              @click="() => { editingOptionId = opt.id; editOptionValue = opt.value }">Edit</el-button>
            <el-button size="small" text type="danger"
              @click="() => { if (confirm(`Remove option '${opt.value}'?`)) removeOption.mutate({ fieldId: f.id, optionId: opt.id }) }">
              Remove
            </el-button>
          </template>
        </div>
        <div style="display:flex;gap:8px;margin-top:8px">
          <el-input v-model="newOptionValue" size="small" placeholder="New option..."
            style="flex:1" @keydown.enter="() => { if (newOptionValue) addOption.mutate({ fieldId: f.id, value: newOptionValue }) }" />
          <el-button size="small" type="primary" :disabled="!newOptionValue || addOption.isPending.value"
            @click="addOption.mutate({ fieldId: f.id, value: newOptionValue })">Add</el-button>
        </div>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/views/FieldsView.vue
git commit -m "feat: add FieldsView with inline edit and enum options"
```

---

### Task 10: Tests

**Files:**
- Create: `frontend/src/tests/LoginView.test.ts`
- Create: `frontend/src/tests/ConflictDialog.test.ts`

- [ ] **Step 1: Write LoginView.test.ts**

Create `frontend/src/tests/LoginView.test.ts`:

```ts
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import ElementPlus from 'element-plus'
import LoginView from '../views/LoginView.vue'

// Mock the auth store login action
const mockLogin = vi.fn()

vi.mock('../stores/auth', () => ({
  useAuthStore: () => ({
    login: mockLogin,
    auth: null,
    isLoading: false,
  }),
}))

function mountLoginView() {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/', component: { template: '<div />' } }],
  })
  return mount(LoginView, {
    global: {
      plugins: [createPinia(), router, ElementPlus],
    },
  })
}

describe('LoginView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    mockLogin.mockReset()
  })

  it('calls login with correct credentials on submit', async () => {
    mockLogin.mockResolvedValueOnce(undefined)
    const wrapper = mountLoginView()

    await wrapper.find('input[autocomplete="username"]').setValue('admin')
    await wrapper.find('input[autocomplete="current-password"]').setValue('secret')
    await wrapper.find('button[type="submit"]').trigger('click')

    expect(mockLogin).toHaveBeenCalledWith('admin', 'secret')
  })

  it('shows error message on failed login', async () => {
    mockLogin.mockRejectedValueOnce(new Error('Unauthorized'))
    const wrapper = mountLoginView()

    await wrapper.find('input[autocomplete="username"]').setValue('admin')
    await wrapper.find('input[autocomplete="current-password"]').setValue('wrong')
    await wrapper.find('button[type="submit"]').trigger('click')

    // wait for async rejection
    await new Promise(r => setTimeout(r, 0))
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Invalid username or password')
  })
})
```

- [ ] **Step 2: Write ConflictDialog.test.ts**

Create `frontend/src/tests/ConflictDialog.test.ts`:

```ts
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import ConflictDialog from '../components/ConflictDialog.vue'
import type { Member } from '../types'

const baseMember: Member = {
  id: '1', version: 2,
  first_name: 'Anna', last_name: 'Mueller',
  email: 'server@example.com', phone: null, street: null, city: null,
  postal_code: null, birth_date: null, membership_type: 'Aktiv',
  joined_at: '2024-01-01', left_at: null, notes: null,
  custom_fields: {}, created_at: '', updated_at: '',
}

describe('ConflictDialog', () => {
  it('renders differing fields', () => {
    const wrapper = mount(ConflictDialog, {
      props: {
        myDraft: { ...baseMember, email: 'mine@example.com' },
        serverMember: baseMember,
        onResolve: vi.fn(),
        onDiscard: vi.fn(),
      },
      global: { plugins: [ElementPlus] },
    })

    expect(wrapper.text()).toContain('mine@example.com')
    expect(wrapper.text()).toContain('server@example.com')
  })

  it('calls onDiscard when "Use server version" is clicked', async () => {
    const onDiscard = vi.fn()
    const wrapper = mount(ConflictDialog, {
      props: {
        myDraft: { ...baseMember, email: 'mine@example.com' },
        serverMember: baseMember,
        onResolve: vi.fn(),
        onDiscard,
      },
      global: { plugins: [ElementPlus] },
    })

    const buttons = wrapper.findAll('button')
    const discardBtn = buttons.find(b => b.text().includes('Use server version'))
    await discardBtn!.trigger('click')
    expect(onDiscard).toHaveBeenCalled()
  })
})
```

- [ ] **Step 3: Run tests**

```bash
cd frontend && npx vitest run
```

Expected: 3 tests pass (2 ConflictDialog, 1 LoginView credentials test — the error test may require adjustment based on Element Plus rendering).

If the error-message test fails due to Element Plus async rendering, update the assertion to poll with `flushPromises`:

```ts
import { flushPromises } from '@vue/test-utils'
// replace: await new Promise(r => setTimeout(r, 0))
// with:
await flushPromises()
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/tests/
git commit -m "test: add LoginView and ConflictDialog tests"
```

---

### Task 11: Final build verification + PR

**Files:** none (verification only)

- [ ] **Step 1: Full build**

```bash
cd frontend && npm run build
```

Expected: `dist/` generated, no TypeScript errors, no Vite build errors.

- [ ] **Step 2: TypeScript check**

```bash
cd frontend && npx tsc --noEmit
```

Expected: 0 errors.

- [ ] **Step 3: Run all tests**

```bash
cd frontend && npx vitest run
```

Expected: all tests pass.

- [ ] **Step 4: Smoke test checklist (manual)**

Start the full stack (`cargo run` in `backend/`, `npm run dev` in `frontend/`):

1. Navigate to `http://localhost:5173` — redirects to `/login`
2. Login → redirects to `/members`, AG Grid renders member rows
3. Click a member row → navigates to `/members/:id`, form loads, presence badge shows
4. Edit a field, Save → success (no conflict)
5. Navigate to `/settings/fields` → fields list renders, enum type has expand toggle
6. Navigate to `/settings/admins` (SuperAdmin only) → admin list renders
7. Logout → redirects to `/login`

- [ ] **Step 5: Create PR**

```bash
gh pr create \
  --title "feat: replace React frontend with Vue 3 + Element Plus" \
  --body "Replaces the React+Tailwind frontend with Vue 3 (Composition API), Element Plus, AG Grid, Pinia, and @tanstack/vue-query at full feature parity. All api/ and types/ modules carried over unchanged."
```
