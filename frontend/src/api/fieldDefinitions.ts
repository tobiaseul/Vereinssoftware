import { client } from './client'
import type { FieldDefinition, FieldType } from '../types'
export const getFieldDefinitions = () => client.get<FieldDefinition[]>('/api/v1/field-definitions').then(r => r.data)
export const createFieldDefinition = (data: { name: string; field_type: FieldType; required?: boolean; display_order?: number }) =>
  client.post<FieldDefinition>('/api/v1/field-definitions', data).then(r => r.data)
export const deleteFieldDefinition = (id: string) => client.delete(`/api/v1/field-definitions/${id}`).then(r => r.data)
