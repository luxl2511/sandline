import { useState, useEffect, useMemo } from 'react'
import { useMapStore } from '@/lib/store'
import { useAuth } from '@/contexts/AuthContext'
import ProposalList from './ProposalList'
import RouteCreationDialog from './RouteCreationDialog'
import AuthModal from '../auth/AuthModal'
import PresenceIndicators from './PresenceIndicators'
import PointChangeProposalList from './PointChangeProposalList' // Import the new component
import type { Route } from '@/types'

interface RouteEditorProps {
  routes: Route[]
}

export default function RouteEditor({ routes }: RouteEditorProps) {
  const {
    isDrawing,
    startDrawing,
    drawnGeometry,
    activeSessions,
    editingRouteId,
    setEditingRouteId,
    clearEditingState,
    pendingPointChanges, // Get pendingPointChanges from the store
  } = useMapStore()
  const { user } = useAuth()
  const [showSaveDialog, setShowSaveDialog] = useState(false)
  const [showAuthModal, setShowAuthModal] = useState(false)

  const editingRoute = useMemo(() => {
    return routes.find(r => r.id === editingRouteId) || null
  }, [routes, editingRouteId])

  const filteredPendingPointChanges = useMemo(() => {
    return pendingPointChanges.filter(
      (change) => change.routeId === editingRouteId && change.status === 'pending'
    )
  }, [pendingPointChanges, editingRouteId])

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

  const handleViewDetails = (routeId: string) => {
    setEditingRouteId(routeId)
  }

  const handleCloseDetails = () => {
    clearEditingState()
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
        ) : !editingRouteId ? ( // No route selected for detail view
          <div className="space-y-2">
            <button
              onClick={handleCreateRoute}
              className="w-full bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 text-sm"
            >
              Create New Route
            </button>
            <p className="text-xs text-gray-600 dark:text-gray-400 mt-2">
              Select a route on the map to view its details or edit.
            </p>
          </div>
        ) : editingRoute ? ( // A route is selected for detail view
          <div className="space-y-4">
            <div>
              <h4 className="font-semibold text-sm text-gray-900 dark:text-gray-50">{editingRoute.name}</h4>
              <p className="text-xs text-gray-600 dark:text-gray-400">
                Created: {new Date(editingRoute.createdAt).toLocaleDateString()}
              </p>
            </div>

            {/* Show presence indicators */}
            <PresenceIndicators sessions={activeSessions} />

            <div className="space-y-2">
              <button className="w-full bg-purple-600 text-white px-4 py-2 rounded hover:bg-purple-700 text-sm">
                Create Proposal
              </button>
              <button
                onClick={handleCloseDetails}
                className="w-full bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-200 px-4 py-2 rounded hover:bg-gray-300 dark:hover:bg-gray-600 text-sm"
              >
                Close Details
              </button>
            </div>

            {editingRouteId && <ProposalList routeId={editingRouteId} />}

            {filteredPendingPointChanges.length > 0 && editingRoute && (
              <PointChangeProposalList
                pointChanges={filteredPendingPointChanges}
                routeOwnerId={editingRoute.ownerId}
                currentUserId={user?.id}
              />
            )}
          </div>
        ) : null}
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
