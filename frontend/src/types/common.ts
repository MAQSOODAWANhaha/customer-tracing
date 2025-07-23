export interface ApiResponse<T = any> {
  data: T
  message?: string
  success?: boolean
}

export interface ApiError {
  message: string
  status?: number
  code?: string
}

export interface PaginationQuery {
  page?: number
  limit?: number
}

export interface PaginationResponse<T> {
  data: T[]
  total: number
  page: number
  limit: number
}

export interface SelectOption {
  label: string
  value: string | number
}

export interface FormField {
  label: string
  key: string
  type: 'text' | 'textarea' | 'number' | 'select' | 'date'
  required?: boolean
  placeholder?: string
  options?: SelectOption[]
  rules?: any[]
}

export interface LoadingState {
  loading: boolean
  error: string | null
}