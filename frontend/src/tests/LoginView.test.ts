import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import ElementPlus from 'element-plus'
import LoginView from '../views/LoginView.vue'

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
    attachTo: document.body,
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
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(mockLogin).toHaveBeenCalledWith('admin', 'secret')
  })

  it('shows error message on failed login', async () => {
    mockLogin.mockRejectedValueOnce(new Error('Unauthorized'))
    const wrapper = mountLoginView()

    await wrapper.find('input[autocomplete="username"]').setValue('admin')
    await wrapper.find('input[autocomplete="current-password"]').setValue('wrong')
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(wrapper.text()).toContain('Invalid username or password')
  })
})
