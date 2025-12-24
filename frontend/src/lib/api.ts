import axios from 'axios'
import { supabase } from './supabase'
import type { CuratedTrack, Route, RouteProposal, EditingSession, PointChange } from '@/types'

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'

const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Add auth interceptor to attach JWT token to all requests
api.interceptors.request.use(async (config) => {
  const { data: { session } } = await supabase.auth.getSession()

  if (session?.access_token) {
    config.headers.Authorization = `Bearer ${session.access_token}`
  }

  return config
})

// Tracks
export async function fetchTracks(
  bounds?: [number, number, number, number]
): Promise<CuratedTrack[]> {
  const params = bounds ? { bbox: bounds.join(',') } : {}
  const response = await api.get('/api/tracks', { params })
  return response.data
}

export async function fetchTrackById(id: string): Promise<CuratedTrack> {
  const response = await api.get(`/api/tracks/${id}`)
  return response.data
}

// Routes
export async function fetchRoutes(): Promise<Route[]> {
  const response = await api.get('/api/routes')
  return response.data
}

export async function fetchRouteById(id: string): Promise<Route> {
  const response = await api.get(`/api/routes/${id}`)
  return response.data
}

export async function createRoute(data: {
  name: string
  geometry: GeoJSON.MultiLineString
}): Promise<Route> {
  const response = await api.post('/api/routes', data)
  return response.data
}

export async function updateRoute(
  id: string,
  data: { geometry: GeoJSON.MultiLineString }
): Promise<Route> {
  const response = await api.put(`/api/routes/${id}`, data)
  return response.data
}

// Proposals
export async function fetchProposals(routeId: string): Promise<RouteProposal[]> {
  const response = await api.get(`/api/routes/${routeId}/proposals`)
  return response.data
}

export async function createProposal(data: {
  routeId: string
  geometry: GeoJSON.MultiLineString
  comment: string
}): Promise<RouteProposal> {
  const response = await api.post('/api/proposals', data)
  return response.data
}

export async function updateProposalStatus(
  id: string,
  status: 'accepted' | 'rejected'
): Promise<RouteProposal> {
  const response = await api.patch(`/api/proposals/${id}`, { status })
  return response.data
}

// Collaborative Editing Sessions
export async function joinEditingSession(
  routeId: string,
  data: {
    userEmail: string
    userAvatarUrl?: string
  }
): Promise<{
  sessionId: string
  routeId: string
  userId: string
  startedAt: string
  activeSessions: EditingSession[]
}> {
  const response = await api.post(`/api/routes/${routeId}/editing-session`, data)
  return response.data
}

export async function leaveEditingSession(routeId: string): Promise<void> {
  await api.delete(`/api/routes/${routeId}/editing-session`)
}

export async function sendHeartbeat(routeId: string): Promise<void> {
  await api.post(`/api/routes/${routeId}/editing-session/heartbeat`)
}

// Point Changes
export async function createPointChange(
  routeId: string,
  data: {
    featureIndex: number
    pointIndex: number
    originalPosition: [number, number]
    newPosition: [number, number]
  }
): Promise<PointChange> {
  const response = await api.post(`/api/routes/${routeId}/point-changes`, data)
  return response.data
}

export async function fetchPointChanges(
  routeId: string,
  status: 'pending' | 'accepted' | 'rejected' = 'pending'
): Promise<PointChange[]> {
  const response = await api.get(`/api/routes/${routeId}/point-changes`, {
    params: { status },
  })
  return response.data
}

export async function updatePointChangeStatus(
  changeId: string,
  status: 'accepted' | 'rejected'
): Promise<PointChange> {
  const response = await api.patch(`/api/point-changes/${changeId}`, { status })
  return response.data
}
