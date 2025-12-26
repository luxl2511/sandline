'use client'

import MapboxMap, { MapRef } from 'react-map-gl'
import { useRef, useCallback, useMemo, useState } from 'react'
import { useMapStore } from '@/lib/store'
import TrackRenderer from './TrackRenderer'
import RouteRenderer from './RouteRenderer'
import ControlPointsLayer from './ControlPointsLayer'
import PointChangesLayer from './PointChangesLayer'
import LiveCursorsLayer from './LiveCursorsLayer'
import SegmentOptionsBubble from './SegmentOptionsBubble' // Import the new component
import RouteStatsPanel from './RouteStatsPanel' // Import the new component
import useMapboxDraw from '@/hooks/useMapboxDraw'
import useEditingSession from '@/hooks/useEditingSession'
import useRealtimePointChanges from '@/hooks/useRealtimePointChanges'
import useCollaborativeMapboxDraw from '@/hooks/useCollaborativeMapboxDraw'
import { useLiveCursors } from '@/hooks/useLiveCursors'
import { useRealtimeBroadcast } from '@/hooks/useRealtimeBroadcast'
import type { Route } from '@/types'

const MAPBOX_TOKEN = process.env.NEXT_PUBLIC_MAPBOX_TOKEN || ''

interface MapViewProps {
  routes: Route[]
}

export default function MapView({ routes }: MapViewProps) {
  const mapRef = useRef<MapRef>(null)
  const {
    layers,
    isDrawing,
    editingRouteId,
    pendingPointChanges,
    setDrawnGeometry,
    setEditingRouteId,
  } = useMapStore()

  const [segmentBubbleInfo, setSegmentBubbleInfo] = useState<({ routeId: string; longitude: number; latitude: number } | null)>(null)

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

  const handleRouteClick = useCallback((routeId: string, coordinates: [number, number]) => {
    setEditingRouteId(routeId)
    setSegmentBubbleInfo({ routeId, longitude: coordinates[0], latitude: coordinates[1] })
  }, [setEditingRouteId])

  const handleCloseSegmentBubble = useCallback(() => {
    setSegmentBubbleInfo(null)
  }, [])

  const handleMapClick = useCallback((e: mapboxgl.MapLayerMouseEvent) => {
    if (isDrawing) return
    const features = e.features || []

    // Check if a route was clicked
    const routeFeature = features.find(f =>
      f.layer?.id === 'routes-casing' ||
      f.layer?.id === 'routes-core'
    )
    if (routeFeature && routeFeature.properties) {
      const clickedRouteId = routeFeature.properties.id
      // Use the actual click coordinates from the event
      const clickedCoords: [number, number] = [e.lngLat.lng, e.lngLat.lat]
      handleRouteClick(clickedRouteId, clickedCoords)
      return
    }

    // If no features clicked, start drawing
    // Logic removed to enforce using the "New Route" button

  }, [isDrawing, handleRouteClick])

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
  })

  useRealtimePointChanges(editingRouteId)

  const editingRoute = useMemo(() => {
    return routes.find(r => r.id === editingRouteId) || null
  }, [routes, editingRouteId])

  useCollaborativeMapboxDraw(mapRef, {
    routeId: editingRouteId,
    currentGeometry: editingRoute?.geometry || null,
    onPointMoved: handlePointMoved,
  })

  // Live cursors tracking
  const { cursors } = useLiveCursors(mapRef, editingRouteId)

  // Live drags from broadcast messages
  const { messages } = useRealtimeBroadcast(editingRouteId)
  const liveDrags = useMemo(() => {
    // Build map of active drags: userId -> drag state
    const dragStates = new Map<string, {
      user_id: string
      user_email: string
      feature_index: number
      point_index: number
      original_position: [number, number]
      new_position: [number, number]
    }>()

    // Process messages chronologically to track drag lifecycle
    messages.forEach((m) => {
      const key = `${m.user_id}-${m.data.feature_index}-${m.data.point_index}`

      if (m.type === 'drag_start') {
        // Start tracking this drag
        dragStates.set(key, {
          user_id: m.user_id,
          user_email: m.user_email,
          feature_index: m.data.feature_index,
          point_index: m.data.point_index,
          original_position: m.data.original_position,
          new_position: m.data.original_position, // Initially same as original
        })
      } else if (m.type === 'drag_update') {
        // Update position if we're tracking this drag
        const existing = dragStates.get(key)
        if (existing) {
          dragStates.set(key, {
            ...existing,
            new_position: m.data.new_position,
          })
        }
      } else if (m.type === 'drag_end') {
        // Remove completed drag
        dragStates.delete(key)
      }
    })

    return Array.from(dragStates.values())
  }, [messages])

  return (
    <MapboxMap
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
      interactiveLayerIds={['routes-casing', 'routes-core']}
      onClick={handleMapClick}
    >
      <TrackRenderer />
      {layers.routes && <RouteRenderer routes={routes} />}
      {layers.routes && <ControlPointsLayer routes={routes} />}

      {/* Show point changes layer when editingRouteId is active */}
      {editingRouteId && (
        <PointChangesLayer changes={pendingPointChanges} liveDrags={liveDrags} />
      )}

      {/* Show live cursors when editingRouteId is active */}
      {editingRouteId && <LiveCursorsLayer cursors={cursors} />}

      {segmentBubbleInfo && (
        <SegmentOptionsBubble
          longitude={segmentBubbleInfo.longitude}
          latitude={segmentBubbleInfo.latitude}
          onClose={handleCloseSegmentBubble}
          segmentLengthKm={10.5} // Placeholder
          estimatedTimeMin={15} // Placeholder
        />
      )}

      {editingRoute && (
        <div className="absolute top-20 right-4 z-10">
          <RouteStatsPanel route={editingRoute} />
        </div>
      )}
    </MapboxMap>
  )
}
