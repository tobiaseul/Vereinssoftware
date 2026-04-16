// frontend/src/App.tsx
import { BrowserRouter, Routes, Route, Navigate, Link } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AuthProvider, useAuth } from './hooks/useAuth'
import { ProtectedRoute } from './components/ProtectedRoute'
import { SuperAdminRoute } from './components/SuperAdminRoute'
import { LoginPage } from './pages/LoginPage'
import { MembersPage } from './pages/MembersPage'
import { MemberDetailPage } from './pages/MemberDetailPage'
import { MemberNewPage } from './pages/MemberNewPage'
import { FieldsPage } from './pages/FieldsPage'
import { AdminsPage } from './pages/AdminsPage'

const queryClient = new QueryClient()

function Nav() {
  const { auth, logout } = useAuth()
  if (!auth) return null
  return (
    <nav className="bg-gray-800 text-white px-6 py-3 flex gap-6 items-center">
      <Link to="/members" className="hover:text-blue-300">Members</Link>
      <Link to="/settings/fields" className="hover:text-blue-300">Fields</Link>
      {auth.role === 'SuperAdmin' && <Link to="/settings/admins" className="hover:text-blue-300">Admins</Link>}
      <button onClick={logout} className="ml-auto text-sm hover:text-red-300">Logout</button>
    </nav>
  )
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <BrowserRouter>
          <Nav />
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/members" element={<ProtectedRoute><MembersPage /></ProtectedRoute>} />
            <Route path="/members/new" element={<ProtectedRoute><MemberNewPage /></ProtectedRoute>} />
            <Route path="/members/:id" element={<ProtectedRoute><MemberDetailPage /></ProtectedRoute>} />
            <Route path="/settings/fields" element={<ProtectedRoute><FieldsPage /></ProtectedRoute>} />
            <Route path="/settings/admins" element={<SuperAdminRoute><AdminsPage /></SuperAdminRoute>} />
            <Route path="*" element={<Navigate to="/members" replace />} />
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </QueryClientProvider>
  )
}
