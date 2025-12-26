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
    <Source id="routes" type="geojson" data={geojson}>
      <Layer
        id="routes-layer"
        type="line"
        paint={{
          'line-color': '#8884d8',
          'line-width': 3,
          'line-opacity': 0.9,
          'line-gradient': [
            'interpolate',
            ['linear'],
            ['line-progress'],
            0,
            'rgba(136, 132, 216, 0)',
            0.5,
            'rgba(136, 132, 216, 1)',
            1,
            'rgba(136, 132, 216, 1)',
          ],
        }}
      />
    </Source>
  )
}
