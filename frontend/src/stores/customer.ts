import { defineStore } from 'pinia'
import { ref } from 'vue'
import request from '@/utils/request'
import type { 
  Customer, 
  CustomerWithLatestTrack,
  CustomerCreateRequest, 
  CustomerUpdateRequest,
  CustomerListResponse,
  NextAction,
  CustomerGroup,
  ApiError
} from '@/types'

interface CustomerFilters {
  page?: number
  limit?: number
  search?: string
  status?: NextAction
  customer_group?: CustomerGroup
}

export const useCustomerStore = defineStore('customer', () => {
  const customers = ref<CustomerWithLatestTrack[]>([])
  const currentCustomer = ref<Customer | null>(null)
  const totalCount = ref(0)
  const loading = ref(false)

  // 获取客户列表
  const fetchCustomers = async (filters: CustomerFilters = {}) => {
    try {
      loading.value = true
      const params = new URLSearchParams()
      
      if (filters.page) params.append('page', filters.page.toString())
      if (filters.limit) params.append('limit', filters.limit.toString())
      if (filters.search) params.append('search', filters.search)
      if (filters.status) params.append('status', filters.status)
      if (filters.customer_group) params.append('customer_group', filters.customer_group)
      
      const response = await request.get<CustomerListResponse>(
        `/api/customers?${params.toString()}`
      )
      
      customers.value = response.data.customers
      totalCount.value = response.data.total
      
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '获取客户列表失败')
    } finally {
      loading.value = false
    }
  }

  // 获取单个客户详情
  const fetchCustomer = async (id: number) => {
    try {
      loading.value = true
      const response = await request.get<Customer>(`/api/customers/${id}`)
      currentCustomer.value = response.data
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '获取客户详情失败')
    } finally {
      loading.value = false
    }
  }

  // 创建客户
  const createCustomer = async (customerData: CustomerCreateRequest) => {
    try {
      loading.value = true
      const response = await request.post<Customer>('/api/customers', customerData)
      
      // 转换为 CustomerWithLatestTrack 格式并添加到本地列表
      const customerWithTrack: CustomerWithLatestTrack = {
        ...response.data,
        track_count: 0,
        latest_track_time: undefined,
        latest_next_action: undefined,
        latest_content: undefined
      }
      customers.value.unshift(customerWithTrack)
      totalCount.value += 1
      
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '创建客户失败')
    } finally {
      loading.value = false
    }
  }

  // 更新客户
  const updateCustomer = async (id: number, customerData: CustomerUpdateRequest) => {
    try {
      loading.value = true
      const response = await request.put<Customer>(`/api/customers/${id}`, customerData)
      
      // 更新本地列表
      const index = customers.value.findIndex(c => c.id === id)
      if (index !== -1) {
        // 保留原有的跟进数据，只更新客户基本信息
        const existingCustomer = customers.value[index]
        customers.value[index] = {
          ...existingCustomer,
          ...response.data
        }
      }
      
      // 更新当前客户
      if (currentCustomer.value?.id === id) {
        currentCustomer.value = response.data
      }
      
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '更新客户失败')
    } finally {
      loading.value = false
    }
  }

  // 删除客户
  const deleteCustomer = async (id: number) => {
    try {
      loading.value = true
      await request.delete(`/api/customers/${id}`)
      
      // 从本地列表中删除
      const index = customers.value.findIndex(c => c.id === id)
      if (index !== -1) {
        customers.value.splice(index, 1)
        totalCount.value -= 1
      }
      
      // 清除当前客户
      if (currentCustomer.value?.id === id) {
        currentCustomer.value = null
      }
      
      return true
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '删除客户失败')
    } finally {
      loading.value = false
    }
  }

  // 获取客户的下一步行动统计
  const getCustomerStats = async () => {
    try {
      const response = await request.get<{
        total: number
        continuing: number
        ended: number
      }>('/api/customers/stats')
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '获取统计数据失败')
    }
  }

  // 批量更新客户状态
  const batchUpdateStatus = async (customerIds: number[], status: NextAction) => {
    try {
      loading.value = true
      const response = await request.post('/api/customers/batch-update-status', {
        customer_ids: customerIds,
        next_action: status
      })
      
      // 更新本地数据
      customers.value.forEach(customer => {
        if (customerIds.includes(customer.id)) {
          customer.next_action = status
          customer.updated_at = new Date().toISOString()
        }
      })
      
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '批量更新失败')
    } finally {
      loading.value = false
    }
  }

  // 清除当前客户
  const clearCurrentCustomer = () => {
    currentCustomer.value = null
  }

  // 清除所有数据
  const clearAll = () => {
    customers.value = []
    currentCustomer.value = null
    totalCount.value = 0
  }

  return {
    customers,
    currentCustomer,
    totalCount,
    loading,
    fetchCustomers,
    fetchCustomer,
    createCustomer,
    updateCustomer,
    deleteCustomer,
    getCustomerStats,
    batchUpdateStatus,
    clearCurrentCustomer,
    clearAll
  }
})