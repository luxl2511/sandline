import axios from 'axios'
import axiosRetry from 'axios-retry'
import type { CuratedTrack, Route, RouteProposal, EditingSession, PointChange } from '@/types'

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'

const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Configure automatic retry logic
axiosRetry(api, {
  retries: 3,
  retryDelay: axiosRetry.exponentialDelay,
  retryCondition: (error) => {
    // Retry on network errors or 5xx server errors
    return axiosRetry.isNetworkOrIdempotentRequestError(error) ||
           (error.response?.status || 0) >= 500
  },
  onRetry: (retryCount, error, requestConfig) => {
    console.log(`Retrying request (${retryCount}/3):`, requestConfig.url)
  },
})

// LocalStorage keys for JWT caching (must match AuthContext)
const JWT_STORAGE_KEY = 'dakar_planner_jwt'
const JWT_EXPIRY_KEY = 'dakar_planner_jwt_expiry'

/**
 * Get cached JWT from localStorage
 *
 * This is a synchronous operation that reads from localStorage,
 * avoiding the need for async Supabase API calls on every request.
 *
 * @returns JWT token if valid and not expired, null otherwise
 */
const getCachedJWT = (): string | null => {
  if (typeof window === 'undefined') return null

  const token = localStorage.getItem(JWT_STORAGE_KEY)
  const expiry = localStorage.getItem(JWT_EXPIRY_KEY)

  if (!token || !expiry) return null

  const expiresAt = parseInt(expiry, 10)
  const now = Math.floor(Date.now() / 1000) // Current time in seconds

  // Check if token is expired (with 60s buffer to account for clock skew)
  if (now >= expiresAt - 60) {
    // Token expired or about to expire - clear it
    localStorage.removeItem(JWT_STORAGE_KEY)
    localStorage.removeItem(JWT_EXPIRY_KEY)
    return null
  }

  return token
}

// Add auth interceptor to attach JWT token to all requests
// PERFORMANCE: Uses synchronous localStorage read instead of async Supabase call
api.interceptors.request.use((config) => {
  const token = getCachedJWT()

  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }

  return config
})

// Add response interceptor for error logging
api.interceptors.response.use(
  (response) => response,
  (error) => {
    // Extract error details
    const status = error.response?.status
    const message = error.response?.data?.message || error.message

    // Log for debugging
    console.error('[API Error]', {
      url: error.config?.url,
      method: error.config?.method,
      status,
      message,
    })

    // Don't throw on client - let calling code handle it
    // This allows per-request error handling while logging all errors
    return Promise.reject(error)
  }
)

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
  controlPoints: GeoJSON.Point[]
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

export async function updateRouteControlPoints(
  id: string,
  data: {
    controlPoints: GeoJSON.Point[]
    featureIndex: number
    pointIndex: number
  }
): Promise<Route> {
  const response = await api.put(`/api/routes/${id}/control-points`, data)
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
