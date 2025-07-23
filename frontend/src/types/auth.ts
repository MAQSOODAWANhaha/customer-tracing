export interface User {
  id: number
  username: string
  name: string
  last_login_at?: string
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  token: string
  expires_in: number
  user: User
}

export interface RefreshTokenResponse {
  token: string
  expires_in: number
}

export interface LogoutResponse {
  message: string
}

export interface AuthState {
  token: string | null
  user: User | null
  isAuthenticated: boolean
}

export interface CurrentUser {
  id: number
  username: string
  name: string
}