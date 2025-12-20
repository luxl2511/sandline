'use client'

import { useEffect } from 'react'
import { useMapStore } from '@/lib/store'
import { fetchProposals } from '@/lib/api'

interface ProposalListProps {
  routeId: string
}

export default function ProposalList({ routeId }: ProposalListProps) {
  const { proposals, setProposals } = useMapStore()

  useEffect(() => {
    fetchProposals(routeId)
      .then(setProposals)
      .catch(console.error)
  }, [routeId, setProposals])

  if (proposals.length === 0) {
    return (
      <div className="mt-4 pt-4 border-t border-gray-200">
        <h4 className="font-semibold text-xs mb-2">Proposals</h4>
        <p className="text-xs text-gray-500">No proposals yet</p>
      </div>
    )
  }

  return (
    <div className="mt-4 pt-4 border-t border-gray-200">
      <h4 className="font-semibold text-xs mb-2">Proposals ({proposals.length})</h4>
      <div className="space-y-2">
        {proposals.map(proposal => (
          <div
            key={proposal.id}
            className="p-2 bg-gray-50 rounded border border-gray-200"
          >
            <p className="text-xs font-medium">{proposal.comment}</p>
            <div className="flex items-center justify-between mt-1">
              <span className={`text-xs px-2 py-0.5 rounded ${
                proposal.status === 'pending'
                  ? 'bg-yellow-100 text-yellow-800'
                  : proposal.status === 'accepted'
                  ? 'bg-green-100 text-green-800'
                  : 'bg-red-100 text-red-800'
              }`}>
                {proposal.status}
              </span>
              <span className="text-xs text-gray-500">
                {new Date(proposal.createdAt).toLocaleDateString()}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
