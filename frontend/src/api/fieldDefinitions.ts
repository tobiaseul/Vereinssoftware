import { client } from './client'
import type { FieldDefinition, FieldOption, FieldType } from '../types'

export const getFieldDefinitions = () =>
  client.get<FieldDefinition[]>('/api/v1/field-definitions').then(r => r.data)

export const createFieldDefinition = (data: { name: string; field_type: FieldType; required?: boolean; display_order?: number }) =>
  client.post<FieldDefinition>('/api/v1/field-definitions', data).then(r => r.data)

export const updateFieldDefinition = (id: string, data: { name?: string; required?: boolean }) =>
  client.put<FieldDefinition>(`/api/v1/field-definitions/${id}`, data).then(r => r.data)

export const deleteFieldDefinition = (id: string) =>
  client.delete(`/api/v1/field-definitions/${id}`).then(r => r.data)

export const addFieldOption = (fieldId: string, data: { value: string; display_order?: number }) =>
  client.post<FieldOption>(`/api/v1/field-definitions/${fieldId}/options`, data).then(r => r.data)

export const updateFieldOption = (fieldId: string, optionId: string, data: { value?: string; display_order?: number }) =>
  client.put<FieldOption>(`/api/v1/field-definitions/${fieldId}/options/${optionId}`, data).then(r => r.data)

export const deleteFieldOption = (fieldId: string, optionId: string) =>
  client.delete(`/api/v1/field-definitions/${fieldId}/options/${optionId}`).then(r => r.data)
