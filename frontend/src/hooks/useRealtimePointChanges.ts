import { useEffect } from 'react'
import { supabase } from '@/lib/supabase'
import { fetchPointChanges } from '@/lib/api'
import { useMapStore } from '@/lib/store'

export default function useRealtimePointChanges(routeId: string | null) {
  const setPendingPointChanges = useMapStore((state) => state.setPendingPointChanges)

  useEffect(() => {
    if (!routeId) return

    // Fetch initial point changes
    const loadPointChanges = async () => {
      try {
        const changes = await fetchPointChanges(routeId, 'pending')
        setPendingPointChanges(changes)
      } catch (error) {
        console.error('Failed to fetch point changes:', error)
      }
    }

    loadPointChanges()

    // Subscribe to route_point_changes table changes for this route
    const channel = supabase
      .channel(`route-point-changes-${routeId}`)
      .on(
        'postgres_changes',
        {
          event: '*', // Listen to all events (INSERT, UPDATE, DELETE)
          schema: 'public',
          table: 'route_point_changes',
          filter: `route_id=eq.${routeId}`,
        },
        (payload) => {
          console.log('Point change detected:', payload)
          // Refetch point changes when any change occurs
          loadPointChanges()
        }
      )
      .subscribe()

    // Cleanup subscription on unmount
    return () => {
      supabase.removeChannel(channel)
    }
  }, [routeId, setPendingPointChanges])
}
