import { useEffect } from 'react'
import { supabase } from '@/lib/supabase'
import { fetchProposals } from '@/lib/api'
import { useMapStore } from '@/lib/store'

export default function useRealtimeProposals(routeId: string) {
  const setProposals = useMapStore((state) => state.setProposals)

  useEffect(() => {
    // Fetch initial proposals
    const loadProposals = async () => {
      try {
        const proposals = await fetchProposals(routeId)
        setProposals(proposals)
      } catch (error) {
        console.error('Failed to fetch proposals:', error)
      }
    }

    loadProposals()

    // Subscribe to route_proposals table changes for this route
    const channel = supabase
      .channel(`route-proposals-${routeId}`)
      .on(
        'postgres_changes',
        {
          event: '*', // Listen to all events (INSERT, UPDATE, DELETE)
          schema: 'public',
          table: 'route_proposals',
          filter: `route_id=eq.${routeId}`,
        },
        (payload) => {
          console.log('Proposal change detected:', payload)
          // Refetch proposals when any change occurs
          loadProposals()
        }
      )
      .subscribe()

    // Cleanup subscription on unmount
    return () => {
      supabase.removeChannel(channel)
    }
  }, [routeId, setProposals])
}
