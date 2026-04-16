import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { describe, it, expect, vi } from 'vitest'

vi.mock('../hooks/useAuth', () => ({
  useAuth: () => ({
    login: vi.fn().mockRejectedValueOnce(new Error('Unauthorized')),
  }),
}))

import { LoginPage } from './LoginPage'

describe('LoginPage', () => {
  it('shows error on failed login', async () => {
    render(<MemoryRouter><LoginPage /></MemoryRouter>)
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: 'admin' } })
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: 'wrong' } })
    fireEvent.click(screen.getByRole('button', { name: /login/i }))
    await waitFor(() => expect(screen.getByText(/invalid username or password/i)).toBeInTheDocument())
  })
})
