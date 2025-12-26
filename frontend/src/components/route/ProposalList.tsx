'use client'

import { useMapStore } from '@/lib/store'
import useRealtimeProposals from '@/hooks/useRealtimeProposals'

interface ProposalListProps {
  routeId: string
}

export default function ProposalList({ routeId }: ProposalListProps) {
  const proposals = useMapStore((state) => state.proposals)

  // Subscribe to realtime updates for this route's proposals
  useRealtimeProposals(routeId)

  if (proposals.length === 0) {
    return (
      <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
        <h4 className="font-semibold text-xs mb-2 text-gray-900 dark:text-gray-50">Proposals</h4>
        <p className="text-xs text-gray-500 dark:text-gray-400">No proposals yet</p>
      </div>
    )
  }

  return (
    <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
      <h4 className="font-semibold text-xs mb-2 text-gray-900 dark:text-gray-50">Proposals ({proposals.length})</h4>
      <div className="space-y-2">
        {proposals.map(proposal => (
          <div
            key={proposal.id}
            className="p-2 bg-gray-50 dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600"
          >
            <p className="text-xs font-medium text-gray-900 dark:text-gray-100">{proposal.comment}</p>
            <div className="flex items-center justify-between mt-1">
              <span className={`text-xs px-2 py-0.5 rounded ${
                proposal.status === 'pending'
                  ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200'
                  : proposal.status === 'accepted'
                  ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-200'
                  : 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-200'
              }`}>
                {proposal.status}
              </span>
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {new Date(proposal.created_at).toLocaleDateString()}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
