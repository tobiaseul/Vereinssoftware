import { Navigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { auth, isLoading } = useAuth()
  if (isLoading) return <div className="p-8 text-center">Loading...</div>
  if (!auth) return <Navigate to="/login" replace />
  return <>{children}</>
}
