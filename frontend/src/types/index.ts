export interface CuratedTrack {
  id: string;
  geometry: GeoJSON.LineString;
  source: "osm" | "rally" | "curated";
  surface: string;
  confidence: 1 | 2 | 3 | 4 | 5;
  last_verified: string | null;
  region: string;
}

export interface Route {
  id: string;
  name: string;
  owner_id: string;
  geometry: GeoJSON.MultiLineString;
  control_points: GeoJSON.Point[];
  length_km?: number;
  estimated_time_min?: number;
  created_by?: string;
  created_at: string;
}

export interface RouteVersion {
  id: string;
  route_id: string;
  geometry: GeoJSON.MultiLineString;
  created_at: string;
}

export interface RouteProposal {
  id: string;
  route_id: string;
  geometry: GeoJSON.MultiLineString;
  comment: string;
  status: "pending" | "accepted" | "rejected";
  created_at: string;
}

export interface EditingSession {
  user_id: string;
  user_email: string;
  user_avatar_url?: string;
  started_at: string;
}

export interface PointChange {
  id: string;
  route_id: string;
  user_id: string;
  user_email: string;
  feature_index: number;
  point_index: number;
  original_position: [number, number];
  new_position: [number, number];
  status: "pending" | "accepted" | "rejected";
  created_at: string;
  updated_at: string;
  resolved_at?: string;
  resolved_by?: string;
}

export interface LayerState {
  osmTracks: boolean;
  curatedTracks: boolean;
  satellite: boolean;
  routes: boolean;
}

export interface MapStore {
  layers: LayerState;
  proposals: RouteProposal[];
  isDrawing: boolean;
  drawnGeometry: GeoJSON.Feature[] | null;

  // Collaborative editing state
  editingRouteId: string | null;
  editingSession: EditingSession | null;
  activeSessions: EditingSession[];
  pendingPointChanges: PointChange[];

  toggleLayer: (layer: keyof LayerState) => void;
  setProposals: (proposals: RouteProposal[]) => void;
  startDrawing: () => void;
  stopDrawing: () => void;
  setDrawnGeometry: (geometry: GeoJSON.Feature[] | null) => void;

  // Collaborative editing actions
  setEditingRouteId: (routeId: string | null) => void;
  clearEditingState: () => void;
  setEditingSession: (session: EditingSession | null) => void;
  setActiveSessions: (sessions: EditingSession[]) => void;
  addPointChange: (change: PointChange) => void;
  updatePointChange: (
    changeId: string,
    status: "accepted" | "rejected",
  ) => void;
  removePointChange: (changeId: string) => void;
  setPendingPointChanges: (changes: PointChange[]) => void;
}
