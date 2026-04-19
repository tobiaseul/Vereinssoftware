import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'

declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth?: boolean
    requiresSuperAdmin?: boolean
  }
}

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
    {
      path: '/finances',
      component: () => import('../views/finances/FinancesView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/finances/accounts',
      component: () => import('../views/finances/AccountListView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/finances/accounts/:id',
      component: () => import('../views/finances/AccountDetailView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/finances/accounts/:id/transactions',
      component: () => import('../views/finances/TransactionListView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/finances/accounts/:id/transactions/new',
      component: () => import('../views/finances/TransactionFormView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/finances/transactions/:id',
      component: () => import('../views/finances/TransactionDetailView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/finances/accounts/:id/reconciliation',
      component: () => import('../views/finances/ReconciliationView.vue'),
      meta: { requiresAuth: true },
    },
    { path: '/:pathMatch(.*)*', redirect: '/members' },
  ],
})

let silentRefreshPromise: Promise<void> | null = null

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (auth.isLoading) {
    if (!silentRefreshPromise) {
      silentRefreshPromise = auth.silentRefresh().finally(() => {
        silentRefreshPromise = null
      })
    }
    await silentRefreshPromise
  }
  if (to.meta.requiresAuth && !auth.auth) return '/login'
  if (to.meta.requiresSuperAdmin && auth.auth?.role !== 'SuperAdmin') return '/members'
})

export default router
