import { Navigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export function SuperAdminRoute({ children }: { children: React.ReactNode }) {
  const { auth, isLoading } = useAuth()
  if (isLoading) return null
  if (!auth) return <Navigate to="/login" replace />
  if (auth.role !== 'SuperAdmin') return <Navigate to="/members" replace />
  return <>{children}</>
}
