'use client'

import { useEffect, useState, useCallback } from 'react'
import toast from 'react-hot-toast'
import ErrorBoundary from '@/components/ErrorBoundary'
import MapView from '@/components/map/MapView'
import LayerControls from '@/components/map/LayerControls'
import RouteEditor from '@/components/route/RouteEditor'
import AuthButton from '@/components/auth/AuthButton'
import useRealtimeRoutes from '@/hooks/useRealtimeRoutes'
import { fetchRoutes } from '@/lib/api'
import type { Route } from '@/types'

export default function Home() {
  const [routes, setRoutes] = useState<Route[]>([])
  const [isLoading, setIsLoading] = useState(true)

  // Fetch routes on mount
  const loadRoutes = useCallback(async () => {
    try {
      setIsLoading(true)
      const data = await fetchRoutes()
      setRoutes(data)
    } catch (error) {
      console.error('Failed to fetch routes:', error)

      const errorMessage = error instanceof Error ? error.message : 'Failed to load routes'
      toast.error(errorMessage, {
        duration: 5000,
        id: 'fetch-routes-error',
      })
    } finally {
      setIsLoading(false)
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
      <ErrorBoundary>
        <MapView routes={routes} />
      </ErrorBoundary>
      {isLoading && (
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4">
            <p className="text-gray-900 dark:text-gray-50">Loading routes...</p>
          </div>
        </div>
      )}
      <div className="absolute top-4 left-4 z-10">
        <LayerControls />
      </div>
      <div className="absolute top-4 left-1/2 -translate-x-1/2 z-10">
        <AuthButton />
      </div>
      <div className="absolute top-4 right-4 z-10">
        <ErrorBoundary>
          <RouteEditor routes={routes} />
        </ErrorBoundary>
      </div>
    </main>
  )
}
