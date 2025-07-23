import { defineStore } from 'pinia'
import { ref } from 'vue'
import request from '@/utils/request'
import type { 
  CustomerTrack, 
  TrackCreateRequest, 
  TrackUpdateRequest,
  TrackListResponse,
  TrackResponse,
  ApiError
} from '@/types'

interface TrackFilters {
  customer_id?: number
  page?: number
  limit?: number
  start_date?: string
  end_date?: string
}

export const useTrackStore = defineStore('track', () => {
  const tracks = ref<CustomerTrack[]>([])
  const currentTrack = ref<CustomerTrack | null>(null)
  const totalCount = ref(0)
  const loading = ref(false)

  // 获取跟进记录列表
  const fetchTracks = async (filters: TrackFilters = {}) => {
    try {
      loading.value = true
      const params = new URLSearchParams()
      
      if (filters.customer_id) params.append('customer_id', filters.customer_id.toString())
      if (filters.page) params.append('page', filters.page.toString())
      if (filters.limit) params.append('limit', filters.limit.toString())
      if (filters.start_date) params.append('start_date', filters.start_date)
      if (filters.end_date) params.append('end_date', filters.end_date)
      
      const response = await request.get<TrackListResponse>(
        `/api/tracks?${params.toString()}`
      )
      
      tracks.value = response.data.tracks
      totalCount.value = response.data.total
      
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '获取跟进记录失败')
    } finally {
      loading.value = false
    }
  }

  // 获取单个跟进记录详情
  const fetchTrack = async (id: number) => {
    try {
      loading.value = true
      const response = await request.get<TrackResponse>(`/api/tracks/${id}`)
      currentTrack.value = response.data.track
      return response.data.track
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '获取跟进记录详情失败')
    } finally {
      loading.value = false
    }
  }

  // 创建跟进记录
  const createTrack = async (trackData: TrackCreateRequest) => {
    try {
      loading.value = true
      const response = await request.post<TrackResponse>('/api/tracks', trackData)
      
      // 添加到本地列表
      tracks.value.unshift(response.data.track)
      totalCount.value += 1
      
      return response.data.track
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '创建跟进记录失败')
    } finally {
      loading.value = false
    }
  }

  // 更新跟进记录
  const updateTrack = async (id: number, trackData: TrackUpdateRequest) => {
    try {
      loading.value = true
      const response = await request.put<TrackResponse>(`/api/tracks/${id}`, trackData)
      
      // 更新本地列表
      const index = tracks.value.findIndex(t => t.id === id)
      if (index !== -1) {
        tracks.value[index] = response.data.track
      }
      
      // 更新当前跟进记录
      if (currentTrack.value?.id === id) {
        currentTrack.value = response.data.track
      }
      
      return response.data.track
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '更新跟进记录失败')
    } finally {
      loading.value = false
    }
  }

  // 删除跟进记录
  const deleteTrack = async (id: number) => {
    try {
      loading.value = true
      await request.delete(`/api/tracks/${id}`)
      
      // 从本地列表中删除
      const index = tracks.value.findIndex(t => t.id === id)
      if (index !== -1) {
        tracks.value.splice(index, 1)
        totalCount.value -= 1
      }
      
      // 清除当前跟进记录
      if (currentTrack.value?.id === id) {
        currentTrack.value = null
      }
      
      return true
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '删除跟进记录失败')
    } finally {
      loading.value = false
    }
  }

  // 获取客户的跟进统计
  const getTrackStats = async (customerId: number) => {
    try {
      const response = await request.get<{
        total: number
        last_track_date: string | null
        avg_days_between_tracks: number
        next_action_stats: {
          continuing: number
          ended: number
        }
      }>(`/api/tracks/stats/${customerId}`)
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '获取跟进统计失败')
    }
  }

  // 批量删除跟进记录
  const batchDeleteTracks = async (trackIds: number[]) => {
    try {
      loading.value = true
      const response = await request.post('/api/tracks/batch-delete', {
        track_ids: trackIds
      })
      
      // 更新本地数据
      tracks.value = tracks.value.filter(track => !trackIds.includes(track.id))
      totalCount.value -= trackIds.length
      
      return response.data
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '批量删除失败')
    } finally {
      loading.value = false
    }
  }

  // 导出跟进记录
  const exportTracks = async (filters: TrackFilters = {}) => {
    try {
      const params = new URLSearchParams()
      
      if (filters.customer_id) params.append('customer_id', filters.customer_id.toString())
      if (filters.start_date) params.append('start_date', filters.start_date)
      if (filters.end_date) params.append('end_date', filters.end_date)
      
      const response = await request.get(
        `/api/tracks/export?${params.toString()}`,
        {
          responseType: 'blob'
        }
      )
      
      // 创建下载链接
      const blob = new Blob([response.data], { 
        type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' 
      })
      const url = window.URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = `tracks_export_${new Date().toISOString().split('T')[0]}.xlsx`
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)
      
      return true
    } catch (error) {
      const apiError = error as ApiError
      throw new Error(apiError.message || '导出失败')
    }
  }

  // 清除当前跟进记录
  const clearCurrentTrack = () => {
    currentTrack.value = null
  }

  // 清除所有数据
  const clearAll = () => {
    tracks.value = []
    currentTrack.value = null
    totalCount.value = 0
  }

  return {
    tracks,
    currentTrack,
    totalCount,
    loading,
    fetchTracks,
    fetchTrack,
    createTrack,
    updateTrack,
    deleteTrack,
    getTrackStats,
    batchDeleteTracks,
    exportTracks,
    clearCurrentTrack,
    clearAll
  }
})