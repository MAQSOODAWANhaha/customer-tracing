export type NextAction = '继续跟进' | '结束跟进'

export interface Customer {
  id: number
  name: string
  phone?: string
  address?: string
  notes?: string
  rate: number
  next_action: NextAction
  user_id: number
  track_count?: number
  last_track_at?: string
  created_at: string
  updated_at: string
  is_deleted: boolean
}

export interface CustomerWithLatestTrack {
  id: number
  name: string
  phone?: string
  address?: string
  rate: number
  notes?: string
  next_action: NextAction
  latest_track_time?: string
  latest_next_action?: NextAction
  latest_content?: string
  track_count: number
  user_id: number
  created_at: string
  updated_at: string
  is_deleted: boolean
}

export interface CustomerCreateRequest {
  name: string
  phone?: string | null
  address?: string | null
  notes?: string | null
  rate?: number
}

export interface CustomerUpdateRequest {
  name?: string
  phone?: string | null
  address?: string | null
  notes?: string | null
  rate?: number
}

export interface CustomerResponse {
  customer: Customer
}

export interface CustomerListResponse {
  customers: CustomerWithLatestTrack[]
  total: number
  page: number
  limit: number
}

export interface CustomerListQuery {
  page?: number
  limit?: number
  search?: string
}

