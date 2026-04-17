import { describe, it, expect, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
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
  it('renders differing fields', async () => {
    const wrapper = mount(ConflictDialog, {
      props: {
        myDraft: { ...baseMember, email: 'mine@example.com' },
        serverMember: baseMember,
        onResolve: vi.fn(),
        onDiscard: vi.fn(),
      },
      global: { plugins: [ElementPlus] },
      attachTo: document.body,
    })

    await flushPromises()

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
      attachTo: document.body,
    })

    await flushPromises()

    const buttons = wrapper.findAll('button')
    const discardBtn = buttons.find(b => b.text().includes('Use server version'))
    await discardBtn!.trigger('click')

    await flushPromises()
    expect(onDiscard).toHaveBeenCalled()
  })
})
