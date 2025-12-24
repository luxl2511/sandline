'use client'

import { useState, useEffect } from 'react'
import { useMapStore } from '@/lib/store'
import { useAuth } from '@/contexts/AuthContext'
import ProposalList from './ProposalList'
import RouteCreationDialog from './RouteCreationDialog'
import AuthModal from '../auth/AuthModal'

export default function RouteEditor() {
  const { selectedRoute, setSelectedRoute, isDrawing, startDrawing, drawnGeometry } = useMapStore()
  const { user } = useAuth()
  const [showSaveDialog, setShowSaveDialog] = useState(false)
  const [showAuthModal, setShowAuthModal] = useState(false)

  // Show save dialog when user finishes drawing
  useEffect(() => {
    if (isDrawing && drawnGeometry && drawnGeometry.length > 0) {
      setShowSaveDialog(true)
    }
  }, [isDrawing, drawnGeometry])

  const handleCreateRoute = () => {
    // Require authentication to create routes
    if (!user) {
      setShowAuthModal(true)
      return
    }
    startDrawing()
  }

  return (
    <>
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4 w-80 max-h-[80vh] overflow-y-auto">
        <h3 className="font-bold text-sm mb-3 text-gray-900 dark:text-gray-50">Route Editor</h3>

        {isDrawing ? (
          <div className="space-y-2">
            <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-3">
              <p className="text-sm font-medium text-blue-900 dark:text-blue-100 mb-2">
                Drawing Mode
              </p>
              <p className="text-xs text-blue-700 dark:text-blue-300">
                Click on the map to add points to your route. Double-click to finish.
              </p>
            </div>
          </div>
        ) : !selectedRoute ? (
          <div className="space-y-2">
            <button
              onClick={handleCreateRoute}
              className="w-full bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 text-sm"
            >
              Create New Route
            </button>
            <p className="text-xs text-gray-600 dark:text-gray-400">
              Select a route or create a new one to start planning
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            <div>
              <h4 className="font-semibold text-sm text-gray-900 dark:text-gray-50">{selectedRoute.name}</h4>
              <p className="text-xs text-gray-600 dark:text-gray-400">
                Created: {new Date(selectedRoute.createdAt).toLocaleDateString()}
              </p>
            </div>

            <div className="space-y-2">
              <button className="w-full bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700 text-sm">
                Edit Route
              </button>
              <button className="w-full bg-purple-600 text-white px-4 py-2 rounded hover:bg-purple-700 text-sm">
                Create Proposal
              </button>
              <button
                onClick={() => setSelectedRoute(null)}
                className="w-full bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-200 px-4 py-2 rounded hover:bg-gray-300 dark:hover:bg-gray-600 text-sm"
              >
                Close
              </button>
            </div>

            <ProposalList routeId={selectedRoute.id} />
          </div>
        )}
      </div>

      {showSaveDialog && (
        <RouteCreationDialog onClose={() => setShowSaveDialog(false)} />
      )}

      {showAuthModal && (
        <AuthModal onClose={() => setShowAuthModal(false)} />
      )}
    </>
  )
}
