import axios, { type AxiosResponse, type AxiosError, type InternalAxiosRequestConfig } from 'axios'
import type { ApiError } from '@/types'

const request = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor - add auth token
request.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    const token = localStorage.getItem(import.meta.env.VITE_TOKEN_STORAGE_KEY || 'customer_tracker_token')
    
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`
    }
    
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor - handle errors globally
request.interceptors.response.use(
  (response: AxiosResponse) => {
    return response
  },
  async (error: AxiosError) => {
    const apiError: ApiError = {
      message: '请求失败',
      status: error.response?.status,
    }

    if (error.response) {
      const status = error.response.status
      const data = error.response.data as any

      switch (status) {
        case 401:
          // Token expired or invalid
          apiError.message = '登录已过期，请重新登录'
          // Clear token and redirect to login
          localStorage.removeItem(import.meta.env.VITE_TOKEN_STORAGE_KEY || 'customer_tracker_token')
          // Don't redirect here as it might cause infinite loops
          // Let the route guard handle redirection
          break
          
        case 403:
          apiError.message = '没有权限执行此操作'
          break
          
        case 404:
          apiError.message = '请求的资源不存在'
          break
          
        case 422:
          apiError.message = data?.message || '请求参数错误'
          break
          
        case 500:
          apiError.message = '服务器内部错误'
          break
          
        default:
          apiError.message = data?.message || `请求失败 (${status})`
      }
    } else if (error.request) {
      apiError.message = '网络连接错误，请检查网络设置'
    } else {
      apiError.message = '请求配置错误'
    }
    
    return Promise.reject(apiError)
  }
)

export default request

// Helper function to handle API responses
export const handleApiResponse = <T>(response: AxiosResponse<T>): T => {
  return response.data
}

// Helper function to handle API errors
export const handleApiError = (error: any): never => {
  if (error.message) {
    throw error
  }
  throw new Error('未知错误')
}