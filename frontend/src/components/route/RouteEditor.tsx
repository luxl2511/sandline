'use client'

import { useState } from 'react'
import { useMapStore } from '@/lib/store'
import ProposalList from './ProposalList'

export default function RouteEditor() {
  const { selectedRoute, setSelectedRoute } = useMapStore()
  const [isCreating, setIsCreating] = useState(false)

  return (
    <div className="bg-white rounded-lg shadow-lg p-4 w-80 max-h-[80vh] overflow-y-auto">
      <h3 className="font-bold text-sm mb-3">Route Editor</h3>

      {!selectedRoute ? (
        <div className="space-y-2">
          <button
            onClick={() => setIsCreating(true)}
            className="w-full bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 text-sm"
          >
            Create New Route
          </button>
          <p className="text-xs text-gray-600">
            Select a route or create a new one to start planning
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          <div>
            <h4 className="font-semibold text-sm">{selectedRoute.name}</h4>
            <p className="text-xs text-gray-600">
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
              className="w-full bg-gray-200 text-gray-700 px-4 py-2 rounded hover:bg-gray-300 text-sm"
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
