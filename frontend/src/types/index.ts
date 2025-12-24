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
  geometry: GeoJSON.MultiLineString
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

export interface LayerState {
  osmTracks: boolean
  curatedTracks: boolean
  satellite: boolean
  routes: boolean
}

export interface MapStore {
  layers: LayerState
  selectedRoute: Route | null
  proposals: RouteProposal[]
  isDrawing: boolean
  drawnGeometry: GeoJSON.Feature[] | null
  toggleLayer: (layer: keyof LayerState) => void
  setSelectedRoute: (route: Route | null) => void
  setProposals: (proposals: RouteProposal[]) => void
  startDrawing: () => void
  stopDrawing: () => void
  setDrawnGeometry: (geometry: GeoJSON.Feature[] | null) => void
}
