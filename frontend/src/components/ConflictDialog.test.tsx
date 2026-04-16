import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { ConflictDialog } from './ConflictDialog'
import type { Member } from '../types'

const base: Member = {
  id: '1', version: 2, first_name: 'Anna', last_name: 'Mueller',
  email: 'server@example.com', phone: null, street: null, city: null,
  postal_code: null, birth_date: null, membership_type: 'Aktiv',
  joined_at: '2024-01-01', left_at: null, notes: null,
  custom_fields: {}, created_at: '', updated_at: '',
}

describe('ConflictDialog', () => {
  it('shows conflicting fields and calls onDiscard', () => {
    const onDiscard = vi.fn()
    render(
      <ConflictDialog
        myDraft={{ ...base, email: 'mine@example.com' }}
        serverMember={base}
        onResolve={vi.fn()}
        onDiscard={onDiscard}
      />
    )
    expect(screen.getByText('mine@example.com')).toBeInTheDocument()
    expect(screen.getByText('server@example.com')).toBeInTheDocument()
    fireEvent.click(screen.getByText(/use server version/i))
    expect(onDiscard).toHaveBeenCalled()
  })
})
