'use client'

import { useState } from 'react'
import { useMapStore } from '@/lib/store'
import ProposalList from './ProposalList'

export default function RouteEditor() {
  const { selectedRoute, setSelectedRoute } = useMapStore()
  const [isCreating, setIsCreating] = useState(false)

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4 w-80 max-h-[80vh] overflow-y-auto">
      <h3 className="font-bold text-sm mb-3 text-gray-900 dark:text-gray-50">Route Editor</h3>

      {!selectedRoute ? (
        <div className="space-y-2">
          <button
            onClick={() => setIsCreating(true)}
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
  )
}
