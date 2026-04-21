import { client } from './client'
import type { Admin, AdminRole } from '../types'

export interface FinanceRole {
  id: string
  name: string
}

export const getAdmins = () => client.get<Admin[]>('/api/v1/admins').then(r => r.data)
export const createAdmin = (data: { username: string; password: string; role?: AdminRole }) =>
  client.post<Admin>('/api/v1/admins', data).then(r => r.data)
export const deleteAdmin = (id: string) => client.delete(`/api/v1/admins/${id}`).then(r => r.data)
export const changePassword = (id: string, password: string) =>
  client.put(`/api/v1/admins/${id}/password`, { password }).then(r => r.data)

// Finance roles
export const getAdminFinanceRoles = (adminId: string) =>
  client.get<FinanceRole[]>(`/api/v1/finance/admins/${adminId}/roles`).then(r => r.data)
export const assignFinanceRole = (adminId: string, roleName: string) =>
  client.post(`/api/v1/finance/admins/${adminId}/roles`, { role_name: roleName }).then(r => r.data)
export const removeFinanceRole = (adminId: string, roleName: string) =>
  client.delete(`/api/v1/finance/admins/${adminId}/roles/${roleName}`).then(r => r.data)
