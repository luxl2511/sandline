'use client'

import { Source, Layer } from 'react-map-gl'
import type { PointChange } from '@/types'

export interface LiveDrag {
  userId: string
  userEmail: string
  featureIndex: number
  pointIndex: number
  originalPosition: [number, number]
  newPosition: [number, number]
}

interface PointChangesLayerProps {
  changes: PointChange[]
  liveDrags?: LiveDrag[]
  onChangeClick?: (change: PointChange) => void
}

export default function PointChangesLayer({
  changes,
  liveDrags = [],
  onChangeClick,
}: PointChangesLayerProps) {
  if (changes.length === 0 && liveDrags.length === 0) return null

  // Generate color from user ID hash
  const getColorFromUserId = (userId: string): string => {
    let hash = 0
    for (let i = 0; i < userId.length; i++) {
      hash = userId.charCodeAt(i) + ((hash << 5) - hash)
    }
    const colors = [
      '#3b82f6', // blue
      '#10b981', // green
      '#8b5cf6', // purple
      '#ec4899', // pink
      '#6366f1', // indigo
      '#eab308', // yellow
      '#ef4444', // red
    ]
    return colors[Math.abs(hash) % colors.length]
  }

  // Create GeoJSON features for ghost markers (new positions)
  const ghostMarkersGeoJSON: GeoJSON.FeatureCollection = {
    type: 'FeatureCollection',
    features: changes.map((change) => ({
      type: 'Feature' as const,
      geometry: {
        type: 'Point' as const,
        coordinates: change.newPosition,
      },
      properties: {
        changeId: change.id,
        userId: change.userId,
        userEmail: change.userEmail,
        color: getColorFromUserId(change.userId),
      },
    })),
  }

  // Create GeoJSON features for original position markers
  const originalMarkersGeoJSON: GeoJSON.FeatureCollection = {
    type: 'FeatureCollection',
    features: changes.map((change) => ({
      type: 'Feature' as const,
      geometry: {
        type: 'Point' as const,
        coordinates: change.originalPosition,
      },
      properties: {
        changeId: change.id,
        userId: change.userId,
        color: getColorFromUserId(change.userId),
      },
    })),
  }

  // Create GeoJSON features for dotted connection lines
  const connectionLinesGeoJSON: GeoJSON.FeatureCollection = {
    type: 'FeatureCollection',
    features: changes.map((change) => ({
      type: 'Feature' as const,
      geometry: {
        type: 'LineString' as const,
        coordinates: [change.originalPosition, change.newPosition],
      },
      properties: {
        changeId: change.id,
        userId: change.userId,
        color: getColorFromUserId(change.userId),
      },
    })),
  }

  // Layer styles (source is provided by parent Source component)
  const originalMarkerLayer = {
    id: 'point-changes-original-markers',
    type: 'circle' as const,
    paint: {
      'circle-radius': 6,
      'circle-color': ['get', 'color'] as any,
      'circle-opacity': 0.5,
      'circle-stroke-width': 2,
      'circle-stroke-color': '#ffffff',
    },
  }

  const ghostMarkerLayer = {
    id: 'point-changes-ghost-markers',
    type: 'circle' as const,
    paint: {
      'circle-radius': 8,
      'circle-color': ['get', 'color'] as any,
      'circle-opacity': 0.9,
      'circle-stroke-width': 2,
      'circle-stroke-color': '#ffffff',
    },
  }

  const connectionLineLayer = {
    id: 'point-changes-connection-lines',
    type: 'line' as const,
    paint: {
      'line-color': ['get', 'color'] as any,
      'line-width': 2,
      'line-dasharray': [2, 2] as any,
      'line-opacity': 0.7,
    },
  }

  // Create GeoJSON for live drags (real-time broadcasting)
  const liveDragLinesGeoJSON: GeoJSON.FeatureCollection = {
    type: 'FeatureCollection',
    features: liveDrags.map((drag) => ({
      type: 'Feature' as const,
      geometry: {
        type: 'LineString' as const,
        coordinates: [drag.originalPosition, drag.newPosition],
      },
      properties: {
        userId: drag.userId,
        userEmail: drag.userEmail,
        color: getColorFromUserId(drag.userId),
      },
    })),
  }

  const liveDragMarkersGeoJSON: GeoJSON.FeatureCollection = {
    type: 'FeatureCollection',
    features: liveDrags.map((drag) => ({
      type: 'Feature' as const,
      geometry: {
        type: 'Point' as const,
        coordinates: drag.newPosition,
      },
      properties: {
        userId: drag.userId,
        userEmail: drag.userEmail,
        color: getColorFromUserId(drag.userId),
      },
    })),
  }

  // Pulsing animation for live drags (different from static point changes)
  const liveDragLineLayer = {
    id: 'live-drag-lines',
    type: 'line' as const,
    paint: {
      'line-color': ['get', 'color'] as any,
      'line-width': 3,
      'line-dasharray': [3, 3] as any,
      'line-opacity': 0.8, // Slightly more visible than point changes
    },
  }

  const liveDragMarkerLayer = {
    id: 'live-drag-markers',
    type: 'circle' as const,
    paint: {
      'circle-radius': 10,
      'circle-color': ['get', 'color'] as any,
      'circle-opacity': 0.6,
      'circle-stroke-width': 3,
      'circle-stroke-color': '#ffffff',
      'circle-stroke-opacity': 1,
    },
  }

  return (
    <>
      {/* Connection lines (render first, so they're below markers) */}
      <Source id="point-changes-connection-lines" type="geojson" data={connectionLinesGeoJSON}>
        <Layer {...connectionLineLayer} />
      </Source>

      {/* Original position markers */}
      <Source id="point-changes-original-markers" type="geojson" data={originalMarkersGeoJSON}>
        <Layer {...originalMarkerLayer} />
      </Source>

      {/* Ghost markers (new positions) */}
      <Source id="point-changes-ghost-markers" type="geojson" data={ghostMarkersGeoJSON}>
        <Layer {...ghostMarkerLayer} />
      </Source>

      {/* LIVE DRAGS: Real-time collaborative dragging */}
      {liveDrags.length > 0 && (
        <>
          <Source id="live-drag-lines" type="geojson" data={liveDragLinesGeoJSON}>
            <Layer {...liveDragLineLayer} />
          </Source>

          <Source id="live-drag-markers" type="geojson" data={liveDragMarkersGeoJSON}>
            <Layer {...liveDragMarkerLayer} />
          </Source>
        </>
      )}
    </>
  )
}
