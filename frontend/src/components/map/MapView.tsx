'use client'

import { useRef, useCallback } from 'react'
import Map, { MapRef } from 'react-map-gl'
import { useMapStore } from '@/lib/store'
import TrackRenderer from './TrackRenderer'
import useMapboxDraw from '@/hooks/useMapboxDraw'

const MAPBOX_TOKEN = process.env.NEXT_PUBLIC_MAPBOX_TOKEN || ''

export default function MapView() {
  const mapRef = useRef<MapRef>(null)
  const { layers, isDrawing, setDrawnGeometry } = useMapStore()

  const handleDrawCreate = useCallback((features: GeoJSON.Feature[]) => {
    setDrawnGeometry(features)
  }, [setDrawnGeometry])

  const handleDrawUpdate = useCallback((features: GeoJSON.Feature[]) => {
    setDrawnGeometry(features)
  }, [setDrawnGeometry])

  const handleDrawDelete = useCallback(() => {
    setDrawnGeometry(null)
  }, [setDrawnGeometry])

  useMapboxDraw(mapRef, {
    enabled: isDrawing,
    onDrawCreate: handleDrawCreate,
    onDrawUpdate: handleDrawUpdate,
    onDrawDelete: handleDrawDelete,
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
    </Map>
  )
}
