import { client } from './client'
import type { Role } from '../types'
export const getRoles = () => client.get<Role[]>('/api/v1/roles').then(r => r.data)
export const createRole = (name: string) => client.post<Role>('/api/v1/roles', { name }).then(r => r.data)
export const deleteRole = (id: string) => client.delete(`/api/v1/roles/${id}`).then(r => r.data)
