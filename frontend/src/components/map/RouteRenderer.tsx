'use client'

import { Layer, Source } from 'react-map-gl'
import type { Route } from '@/types'

interface RouteRendererProps {
  routes: Route[]
}

export default function RouteRenderer({ routes }: RouteRendererProps) {
  const geojson: GeoJSON.FeatureCollection = {
    type: 'FeatureCollection',
    features: routes
      .filter(route => route.geometry) // Filter out routes with no geometry
      .map(route => ({
        type: 'Feature',
        properties: {
          id: route.id,
          name: route.name,
        },
        geometry: route.geometry!,
      })),
  }

  return (
    <Source id="routes" type="geojson" data={geojson} lineMetrics={true}>
      {/* Outer glow layer */}
      <Layer
        id="routes-glow"
        type="line"
        paint={{
          'line-color': '#22d3ee', // Cyan glow
          'line-width': 8,
          'line-opacity': 0.3,
          'line-blur': 4,
        }}
      />

      {/* Main route layer with gradient */}
      <Layer
        id="routes-layer"
        type="line"
        paint={{
          'line-color': '#06b6d4', // Vibrant cyan
          'line-width': 5,
          'line-opacity': 0.95,
          'line-gradient': [
            'interpolate',
            ['linear'],
            ['line-progress'],
            0,
            '#8b5cf6', // Purple start
            0.5,
            '#06b6d4', // Cyan middle
            1,
            '#10b981', // Green end
          ],
        }}
      />

      {/* Inner highlight for 3D effect */}
      <Layer
        id="routes-highlight"
        type="line"
        paint={{
          'line-color': '#ffffff',
          'line-width': 1.5,
          'line-opacity': 0.6,
        }}
      />
    </Source>
  )
}
