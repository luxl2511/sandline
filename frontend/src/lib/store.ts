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
}))
