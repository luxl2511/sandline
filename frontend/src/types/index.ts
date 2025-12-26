export interface CuratedTrack {
  id: string
  geometry: GeoJSON.LineString
  source: 'osm' | 'rally' | 'curated'
  surface: string
  confidence: 1 | 2 | 3 | 4 | 5
  lastVerified: string | null
  region: string
}

export interface Route {
  id: string
  name: string
  ownerId: string
  geometry: GeoJSON.MultiLineString | null
  controlPoints: GeoJSON.Point[] | null
  createdAt: string
}

export interface RouteVersion {
  id: string
  routeId: string
  geometry: GeoJSON.MultiLineString
  createdAt: string
}

export interface RouteProposal {
  id: string
  routeId: string
  geometry: GeoJSON.MultiLineString
  comment: string
  status: 'pending' | 'accepted' | 'rejected'
  createdAt: string
}

export interface EditingSession {
  userId: string
  userEmail: string
  userAvatarUrl?: string
  startedAt: string
}

export interface PointChange {
  id: string
  routeId: string
  userId: string
  userEmail: string
  featureIndex: number
  pointIndex: number
  originalPosition: [number, number]
  newPosition: [number, number]
  status: 'pending' | 'accepted' | 'rejected'
  createdAt: string
  updatedAt: string
  resolvedAt?: string
  resolvedBy?: string
}

export interface LayerState {
  osmTracks: boolean
  curatedTracks: boolean
  satellite: boolean
  routes: boolean
}

export interface MapStore {
  layers: LayerState
  proposals: RouteProposal[]
  isDrawing: boolean
  drawnGeometry: GeoJSON.Feature[] | null

  // Collaborative editing state
  editingRouteId: string | null
  editingSession: EditingSession | null
  activeSessions: EditingSession[]
  pendingPointChanges: PointChange[]

  toggleLayer: (layer: keyof LayerState) => void
  setProposals: (proposals: RouteProposal[]) => void
  startDrawing: () => void
  stopDrawing: () => void
  setDrawnGeometry: (geometry: GeoJSON.Feature[] | null) => void

  // Collaborative editing actions
  setEditingRouteId: (routeId: string | null) => void
  clearEditingState: () => void
  setEditingSession: (session: EditingSession | null) => void
  setActiveSessions: (sessions: EditingSession[]) => void
  addPointChange: (change: PointChange) => void
  updatePointChange: (changeId: string, status: 'accepted' | 'rejected') => void
  removePointChange: (changeId: string) => void
  setPendingPointChanges: (changes: PointChange[]) => void
}
