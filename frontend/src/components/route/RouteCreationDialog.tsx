'use client'

import { useState } from 'react'
import { useMapStore } from '@/lib/store'
import { createRoute } from '@/lib/api'

interface RouteCreationDialogProps {
  onClose: () => void
}

export default function RouteCreationDialog({ onClose }: RouteCreationDialogProps) {
  const [name, setName] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const { drawnGeometry, stopDrawing } = useMapStore()

  const handleSave = async () => {
    if (!name.trim()) {
      setError('Please enter a route name')
      return
    }

    if (!drawnGeometry || drawnGeometry.length === 0) {
      setError('Please draw a route on the map')
      return
    }

    setIsSubmitting(true)
    setError(null)

    try {
      // Convert drawn features to MultiLineString
      const coordinates = drawnGeometry
        .filter(f => f.geometry.type === 'LineString')
        .map(f => (f.geometry as GeoJSON.LineString).coordinates)

      if (coordinates.length === 0) {
        throw new Error('No valid line strings drawn')
      }

      const geometry: GeoJSON.MultiLineString = {
        type: 'MultiLineString',
        coordinates,
      }

      const controlPoints: GeoJSON.Point[] = coordinates.flat().map(coord => ({
        type: 'Point',
        coordinates: coord,
      }))

      await createRoute({ name: name.trim(), geometry, controlPoints })

      // Success - close dialog and stop drawing
      stopDrawing()
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create route')
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleCancel = () => {
    stopDrawing()
    onClose()
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 w-96">
        <h3 className="text-lg font-bold mb-4 text-gray-900 dark:text-gray-50">
          Save Route
        </h3>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
              Route Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="e.g., Dakar Stage 5"
              autoFocus
            />
          </div>

          {error && (
            <div className="text-sm text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-2 rounded">
              {error}
            </div>
          )}

          <div className="flex space-x-3">
            <button
              onClick={handleSave}
              disabled={isSubmitting}
              className="flex-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white px-4 py-2 rounded text-sm font-medium"
            >
              {isSubmitting ? 'Saving...' : 'Save Route'}
            </button>
            <button
              onClick={handleCancel}
              disabled={isSubmitting}
              className="flex-1 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-200 px-4 py-2 rounded text-sm font-medium"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
