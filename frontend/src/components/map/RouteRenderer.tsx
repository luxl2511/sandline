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
      {/* 1. Dark "Casing" / Shadow Layer - provides contrast against map */}
      <Layer
        id="routes-casing"
        type="line"
        paint={{
          'line-color': '#0f172a', // Slate 900
          'line-width': 12,
          'line-opacity': 0.7,
          'line-blur': 2,
        }}
        layout={{
          'line-cap': 'round',
          'line-join': 'round',
        }}
      />

      {/* 2. Outer Neon Glow - Wide and blurry */}
      <Layer
        id="routes-glow"
        type="line"
        paint={{
          'line-color': '#8b5cf6', // Violet glow
          'line-width': 16,
          'line-opacity': 0.4,
          'line-blur': 12,
        }}
        layout={{
          'line-cap': 'round',
          'line-join': 'round',
        }}
      />

      {/* 3. Inner Neon Glow - Tighter and brighter */}
      <Layer
        id="routes-inner-glow"
        type="line"
        paint={{
          'line-color': '#06b6d4', // Cyan glow
          'line-width': 8,
          'line-opacity': 0.6,
          'line-blur': 4,
        }}
        layout={{
          'line-cap': 'round',
          'line-join': 'round',
        }}
      />

      {/* 4. Core Line - Sharp gradient */}
      <Layer
        id="routes-core"
        type="line"
        paint={{
          'line-width': 4,
          'line-gradient': [
            'interpolate',
            ['linear'],
            ['line-progress'],
            0,
            '#ef4444', // Red (Start)
            0.2,
            '#f472b6', // Pink
            0.5,
            '#a855f7', // Purple
            0.8,
            '#06b6d4', // Cyan
            1,
            '#10b981', // Emerald (End)
          ],
        }}
        layout={{
          'line-cap': 'round',
          'line-join': 'round',
        }}
      />

      {/* 5. "Data Flow" Pattern - Dashed line overlay */}
      <Layer
        id="routes-flow"
        type="line"
        paint={{
          'line-color': '#ffffff',
          'line-width': 2,
          'line-opacity': 0.5,
          'line-dasharray': [1, 4], // Dots
        }}
        layout={{
          'line-cap': 'round',
          'line-join': 'round',
        }}
      />
    </Source>
  )
}
