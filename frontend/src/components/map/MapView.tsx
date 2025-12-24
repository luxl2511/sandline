'use client'

import { useRef, useCallback } from 'react'
import Map, { MapRef } from 'react-map-gl'
import { useMapStore } from '@/lib/store'
import TrackRenderer from './TrackRenderer'
import PointChangesLayer from './PointChangesLayer'
import useMapboxDraw from '@/hooks/useMapboxDraw'
import useEditingSession from '@/hooks/useEditingSession'
import useRealtimePointChanges from '@/hooks/useRealtimePointChanges'
import useCollaborativeMapboxDraw from '@/hooks/useCollaborativeMapboxDraw'

const MAPBOX_TOKEN = process.env.NEXT_PUBLIC_MAPBOX_TOKEN || ''

export default function MapView() {
  const mapRef = useRef<MapRef>(null)
  const {
    layers,
    isDrawing,
    isEditingRoute,
    editingRouteId,
    selectedRoute,
    pendingPointChanges,
    setDrawnGeometry,
  } = useMapStore()

  const handleDrawCreate = useCallback((features: GeoJSON.Feature[]) => {
    setDrawnGeometry(features)
  }, [setDrawnGeometry])

  const handleDrawUpdate = useCallback((features: GeoJSON.Feature[]) => {
    setDrawnGeometry(features)
  }, [setDrawnGeometry])

  const handleDrawDelete = useCallback(() => {
    setDrawnGeometry(null)
  }, [setDrawnGeometry])

  const handlePointMoved = useCallback((change: {
    featureIndex: number
    pointIndex: number
    originalPosition: [number, number]
    newPosition: [number, number]
  }) => {
    console.log('Point moved:', change)
    // Point change is already submitted to API by useCollaborativeMapboxDraw
  }, [])

  // Use regular draw hook for route creation
  useMapboxDraw(mapRef, {
    enabled: isDrawing,
    onDrawCreate: handleDrawCreate,
    onDrawUpdate: handleDrawUpdate,
    onDrawDelete: handleDrawDelete,
  })

  // Use collaborative editing hooks when in edit mode
  useEditingSession({
    routeId: editingRouteId,
    enabled: isEditingRoute,
  })

  useRealtimePointChanges(editingRouteId)

  useCollaborativeMapboxDraw(mapRef, {
    enabled: isEditingRoute,
    routeId: editingRouteId,
    currentGeometry: selectedRoute?.geometry || null,
    onPointMoved: handlePointMoved,
  })

  return (
    <Map
      ref={mapRef}
      mapboxAccessToken={MAPBOX_TOKEN}
      initialViewState={{
        longitude: -5.0,
        latitude: 20.0,
        zoom: 5
      }}
      style={{ width: '100%', height: '100%' }}
      mapStyle={
        layers.satellite
          ? 'mapbox://styles/mapbox/satellite-streets-v12'
          : 'mapbox://styles/mapbox/outdoors-v12'
      }
    >
      <TrackRenderer />

      {/* Show point changes layer when in editing mode */}
      {isEditingRoute && <PointChangesLayer changes={pendingPointChanges} />}
    </Map>
  )
}
