export type AdminRole = 'Admin' | 'SuperAdmin'
export type MembershipType = 'Aktiv' | 'Passiv' | 'Ehrenmitglied'
export type FieldType = 'text' | 'number' | 'date' | 'boolean'

export interface Admin {
  id: string
  username: string
  role: AdminRole
  created_at: string
}

export interface Member {
  id: string
  version: number
  first_name: string
  last_name: string
  email: string | null
  phone: string | null
  street: string | null
  city: string | null
  postal_code: string | null
  birth_date: string | null
  membership_type: MembershipType
  joined_at: string
  left_at: string | null
  notes: string | null
  custom_fields: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface Role {
  id: string
  name: string
  created_at: string
}

export interface FieldDefinition {
  id: string
  name: string
  field_type: FieldType
  required: boolean
  display_order: number
  created_at: string
}

export interface ConflictError {
  code: 'CONFLICT'
  message: string
  details: {
    current_version: number
    submitted_version: number
  }
}

export interface AuthState {
  access_token: string
  admin_id: string
  role: AdminRole
}
