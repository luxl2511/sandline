'use client'

import { useEffect, useState } from 'react'
import { Layer, Source } from 'react-map-gl'
import { useMapStore } from '@/lib/store'
import { fetchTracks } from '@/lib/api'
import type { CuratedTrack } from '@/types'

export default function TrackRenderer() {
  const { layers } = useMapStore()
  const [tracks, setTracks] = useState<CuratedTrack[]>([])

  useEffect(() => {
    if (layers.osmTracks || layers.curatedTracks) {
      fetchTracks().then(setTracks).catch(console.error)
    }
  }, [layers.osmTracks, layers.curatedTracks])

  const geojson: GeoJSON.FeatureCollection = {
    type: 'FeatureCollection',
    features: tracks.map(track => ({
      type: 'Feature',
      properties: {
        id: track.id,
        confidence: track.confidence,
        source: track.source,
        surface: track.surface,
      },
      geometry: track.geometry,
    })),
  }

  if (!layers.osmTracks && !layers.curatedTracks) return null

  return (
    <Source id="tracks" type="geojson" data={geojson}>
      <Layer
        id="tracks-layer"
        type="line"
        paint={{
          'line-color': [
            'match',
            ['get', 'confidence'],
            5, '#22c55e',  // High confidence - green
            4, '#84cc16',  // Good confidence - lime
            3, '#eab308',  // Medium confidence - yellow
            2, '#f97316',  // Low confidence - orange
            1, '#ef4444',  // Very low confidence - red
            '#6b7280'      // Default - gray
          ],
          'line-width': 2,
          'line-opacity': 0.8,
        }}
      />
    </Source>
  )
}
