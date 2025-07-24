import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import request from '@/utils/request'
import type { 
  User, 
  LoginRequest, 
  LoginResponse, 
  RefreshTokenResponse,
  LogoutResponse,
  ApiError 
} from '@/types'

const TOKEN_KEY = 'customer_tracker_token'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem(TOKEN_KEY))
  const user = ref<User | null>(null)
  const loading = ref(false)

  const isAuthenticated = computed(() => !!token.value && !!user.value)

  // Initialize auth state from localStorage
  const initAuth = async (): Promise<boolean> => {
    const savedToken = localStorage.getItem(TOKEN_KEY)
    if (savedToken) {
      token.value = savedToken
      try {
        const response = await request.get<User>('/api/auth/me')
        user.value = response.data
        return true
      } catch (error) {
        // Token is invalid, clear it
        logout()
        return false
      }
    }
    return false
  }

  // Login
  const login = async (credentials: LoginRequest) => {
    try {
      loading.value = true
      const response = await request.post<LoginResponse>('/api/auth/login', credentials)
      
      token.value = response.data.token
      user.value = response.data.user
      
      // Save to localStorage
      localStorage.setItem(TOKEN_KEY, response.data.token)
      
      return { success: true, user: response.data.user }
    } catch (error: any) {
      const apiError = error as ApiError
      return { 
        success: false, 
        message: apiError.message || '登录失败' 
      }
    } finally {
      loading.value = false
    }
  }

  // Logout
  const logout = async () => {
    try {
      if (token.value) {
        await request.post<LogoutResponse>('/api/auth/logout')
      }
    } catch (error) {
      // Ignore logout API errors
    } finally {
      token.value = null
      user.value = null
      localStorage.removeItem(TOKEN_KEY)
    }
  }

  // Refresh token
  const refreshToken = async (): Promise<boolean> => {
    try {
      const response = await request.post<RefreshTokenResponse>('/api/auth/refresh')
      token.value = response.data.token
      localStorage.setItem(TOKEN_KEY, response.data.token)
      return true
    } catch (error) {
      logout()
      return false
    }
  }

  // Get current user info
  const getCurrentUser = async (): Promise<User | null> => {
    try {
      const response = await request.get<User>('/api/auth/me')
      user.value = response.data
      return response.data
    } catch (error) {
      logout()
      return null
    }
  }

  return {
    token,
    user,
    loading,
    isAuthenticated,
    initAuth,
    login,
    logout,
    refreshToken,
    getCurrentUser
  }
})