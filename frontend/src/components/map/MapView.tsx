'use client'

import { useRef, useEffect } from 'react'
import Map, { MapRef, Layer, Source } from 'react-map-gl'
import { useMapStore } from '@/lib/store'
import TrackRenderer from './TrackRenderer'

const MAPBOX_TOKEN = process.env.NEXT_PUBLIC_MAPBOX_TOKEN || ''

export default function MapView() {
  const mapRef = useRef<MapRef>(null)
  const { layers } = useMapStore()

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
    </Map>
  )
}
