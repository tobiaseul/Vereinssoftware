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
