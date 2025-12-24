'use client'

import { useEffect, useState, useCallback } from 'react'
import MapView from '@/components/map/MapView'
import LayerControls from '@/components/map/LayerControls'
import RouteEditor from '@/components/route/RouteEditor'
import AuthButton from '@/components/auth/AuthButton'
import useRealtimeRoutes from '@/hooks/useRealtimeRoutes'
import { fetchRoutes } from '@/lib/api'
import type { Route } from '@/types'

export default function Home() {
  const [routes, setRoutes] = useState<Route[]>([])

  // Fetch routes on mount
  const loadRoutes = useCallback(async () => {
    try {
      const data = await fetchRoutes()
      setRoutes(data)
    } catch (error) {
      console.error('Failed to fetch routes:', error)
    }
  }, [])

  useEffect(() => {
    loadRoutes()
  }, [loadRoutes])

  // Subscribe to realtime route changes
  useRealtimeRoutes({
    onRoutesChange: loadRoutes,
  })

  return (
    <main className="relative w-full h-screen">
      <MapView />
      <div className="absolute top-4 left-4 z-10">
        <LayerControls />
      </div>
      <div className="absolute top-4 left-1/2 -translate-x-1/2 z-10">
        <AuthButton />
      </div>
      <div className="absolute top-4 right-4 z-10">
        <RouteEditor />
      </div>
    </main>
  )
}
