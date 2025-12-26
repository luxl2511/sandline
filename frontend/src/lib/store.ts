import { create } from 'zustand'
import type { MapStore } from '@/types'

export const useMapStore = create<MapStore>((set) => ({
  layers: {
    osmTracks: true,
    curatedTracks: true,
    satellite: false,
    routes: true,
  },
  proposals: [],
  isDrawing: false,
  drawnGeometry: null,

  // Collaborative editing state
  editingRouteId: null, // This now defines the 'active' route for collaboration
  editingSession: null,
  activeSessions: [],
  pendingPointChanges: [],

  toggleLayer: (layer) =>
    set((state) => ({
      layers: {
        ...state.layers,
        [layer]: !state.layers[layer],
      },
    })),
  setProposals: (proposals) =>
    set({ proposals }),
  startDrawing: () =>
    set({ isDrawing: true, drawnGeometry: null }),
  stopDrawing: () =>
    set({ isDrawing: false }),
  setDrawnGeometry: (geometry) =>
    set({ drawnGeometry: geometry }),

  // Collaborative editing actions (simplified)
  setEditingRouteId: (routeId) =>
    set({ editingRouteId: routeId }), // New action to set the active route for collaboration
  clearEditingState: () =>
    set({
      editingRouteId: null,
      editingSession: null,
      activeSessions: [],
      pendingPointChanges: [],
    }),
  setEditingSession: (session) =>
    set({ editingSession: session }),
  setActiveSessions: (sessions) =>
    set({ activeSessions: sessions }),
  addPointChange: (change) =>
    set((state) => ({
      pendingPointChanges: [...state.pendingPointChanges, change],
    })),
  updatePointChange: (changeId, status) =>
    set((state) => ({
      pendingPointChanges: state.pendingPointChanges.map((change) =>
        change.id === changeId ? { ...change, status } : change
      ),
    })),
  removePointChange: (changeId) =>
    set((state) => ({
      pendingPointChanges: state.pendingPointChanges.filter(
        (change) => change.id !== changeId
      ),
    })),
  setPendingPointChanges: (changes) =>
    set({ pendingPointChanges: changes }),
}))
