import { client } from './client'
import type { Member } from '../types'

export interface MemberListParams { search?: string; membership_type?: string; include_left?: boolean }

export const getMembers = (params?: MemberListParams) =>
  client.get<Member[]>('/api/v1/members', { params }).then(r => r.data)

export const getMember = (id: string) =>
  client.get<Member>(`/api/v1/members/${id}`).then(r => r.data)

export const createMember = (data: Partial<Member>) =>
  client.post<Member>('/api/v1/members', data).then(r => r.data)

export const updateMember = (id: string, data: Partial<Member>) =>
  client.put<Member>(`/api/v1/members/${id}`, data).then(r => r.data)

export const deleteMember = (id: string) =>
  client.delete(`/api/v1/members/${id}`).then(r => r.data)

export const exportMembers = (params?: MemberListParams) => {
  const qs = new URLSearchParams(params as any).toString()
  window.open(`/api/v1/members/export${qs ? '?' + qs : ''}`)
}
