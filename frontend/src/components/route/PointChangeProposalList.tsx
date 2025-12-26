'use client'

import { useMemo } from 'react'
import type { PointChange } from '@/types'
import { updatePointChangeStatus } from '@/lib/api'

interface PointChangeProposalListProps {
  pointChanges: PointChange[]
  routeOwnerId: string
  currentUserId: string | undefined
}

export default function PointChangeProposalList({
  pointChanges,
  routeOwnerId,
  currentUserId,
}: PointChangeProposalListProps) {
  const isRouteOwner = useMemo(() => currentUserId === routeOwnerId, [currentUserId, routeOwnerId])

  const handleAccept = async (changeId: string) => {
    try {
      await updatePointChangeStatus(changeId, 'accepted')
    } catch (error) {
      console.error('Failed to accept point change:', error)
      alert('Failed to accept point change.')
    }
  }

  const handleReject = async (changeId: string) => {
    try {
      await updatePointChangeStatus(changeId, 'rejected')
    } catch (error) {
      console.error('Failed to reject point change:', error)
      alert('Failed to reject point change.')
    }
  }

  if (pointChanges.length === 0) return null

  return (
    <div className="mt-4 border-t border-gray-200 dark:border-gray-700 pt-4">
      <h4 className="font-semibold text-sm mb-2 text-gray-900 dark:text-gray-50">
        Pending Point Change Proposals
      </h4>
      <ul className="divide-y divide-gray-200 dark:divide-gray-700">
        {pointChanges.map((change) => (
          <li key={change.id} className="py-2 text-sm">
            <p className="text-gray-700 dark:text-gray-300">
              <span className="font-medium">{change.user_email.split('@')[0]}</span> proposed a change to point{' '}
              <span className="font-mono text-xs bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded">
                [{change.feature_index}, {change.point_index}]
              </span>
            </p>
            <div className="flex space-x-2 mt-2">
              <span className="text-xs text-gray-500 dark:text-gray-400">
                Status: {change.status}
              </span>
              {isRouteOwner && (
                <>
                  <button
                    onClick={() => handleAccept(change.id)}
                    className="bg-green-500 hover:bg-green-600 text-white px-3 py-1 rounded text-xs"
                  >
                    Accept
                  </button>
                  <button
                    onClick={() => handleReject(change.id)}
                    className="bg-red-500 hover:bg-red-600 text-white px-3 py-1 rounded text-xs"
                  >
                    Reject
                  </button>
                </>
              )}
            </div>
          </li>
        ))}
      </ul>
    </div>
  )
}
