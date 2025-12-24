import { create } from 'zustand'
import type { MapStore } from '@/types'

export const useMapStore = create<MapStore>((set) => ({
  layers: {
    osmTracks: true,
    curatedTracks: true,
    satellite: false,
    routes: true,
  },
  selectedRoute: null,
  proposals: [],
  isDrawing: false,
  drawnGeometry: null,

  // Collaborative editing state
  isEditingRoute: false,
  editingRouteId: null,
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
  setSelectedRoute: (route) =>
    set({ selectedRoute: route }),
  setProposals: (proposals) =>
    set({ proposals }),
  startDrawing: () =>
    set({ isDrawing: true, drawnGeometry: null, selectedRoute: null }),
  stopDrawing: () =>
    set({ isDrawing: false }),
  setDrawnGeometry: (geometry) =>
    set({ drawnGeometry: geometry }),

  // Collaborative editing actions
  startEditingRoute: (routeId) =>
    set({
      isEditingRoute: true,
      editingRouteId: routeId,
      isDrawing: false,
    }),
  stopEditingRoute: () =>
    set({
      isEditingRoute: false,
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
