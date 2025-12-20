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

  const getTrackColor = (confidence: number): string => {
    const colors = {
      5: '#22c55e',
      4: '#84cc16',
      3: '#eab308',
      2: '#f97316',
      1: '#ef4444',
    }
    return colors[confidence as keyof typeof colors] || '#6b7280'
  }

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
          'line-color': ['get', 'confidence'],
          'line-width': 2,
          'line-opacity': 0.8,
        }}
      />
    </Source>
  )
}
