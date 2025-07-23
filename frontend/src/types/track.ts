import type { NextAction } from './customer'

export interface CustomerTrack {
  id: number
  customer_id: number
  content: string
  next_action: NextAction
  track_time: string
  next_track_time?: string
  created_at: string
  updated_at: string
}

export interface TrackCreateRequest {
  customer_id: number
  content: string
  next_action: NextAction
}

export interface TrackUpdateRequest {
  content?: string
  next_action?: NextAction
}

export interface TrackResponse {
  track: CustomerTrack
}

export interface TrackListResponse {
  tracks: CustomerTrack[]
  total: number
  page: number
  limit: number
}

export interface CustomerInfo {
  id: number
  name: string
  phone?: string
  rate: number
}

export interface CustomerTrackListResponse {
  tracks: CustomerTrack[]
  customer: CustomerInfo
}

export interface NextActionsResponse {
  actions: string[]
}