import { useEffect, useRef } from 'react'
import { supabase } from '@/lib/supabase'
import { joinEditingSession, leaveEditingSession, sendHeartbeat } from '@/lib/api'
import { useMapStore } from '@/lib/store'
import { useAuth } from '@/contexts/AuthContext'
import type { EditingSession } from '@/types'

interface UseEditingSessionOptions {
  routeId: string | null
}

export default function useEditingSession({ routeId }: UseEditingSessionOptions) {
  const { user } = useAuth()
  const setEditingSession = useMapStore((state) => state.setEditingSession)
  const setActiveSessions = useMapStore((state) => state.setActiveSessions)
  const heartbeatIntervalRef = useRef<NodeJS.Timeout | null>(null)

  useEffect(() => {
    if (!routeId || !user) return

    let channel: ReturnType<typeof supabase.channel> | null = null

    const initialize = async () => {
      try {
        // Join editing session
        const response = await joinEditingSession(routeId, {
          user_email: user.email || '',
          user_avatar_url: user.user_metadata?.avatar_url,
        })

        setEditingSession({
          user_id: response.user_id,
          user_email: user.email || '',
          user_avatar_url: user.user_metadata?.avatar_url,
          started_at: response.started_at,
        })

        setActiveSessions(response.active_sessions)

        // Subscribe to Supabase Presence for real-time session tracking
        channel = supabase.channel(`route-editing:${routeId}`, {
          config: {
            presence: {
              key: user.id,
            },
          },
        })

        // Track presence state
        channel
          .on('presence', { event: 'sync' }, () => {
            const state = channel!.presenceState()
            const sessions: EditingSession[] = Object.values(state)
              .flat()
              .map((presence: any) => ({
                user_id: presence.user_id,
                user_email: presence.user_email,
                user_avatar_url: presence.user_avatar_url,
                started_at: presence.started_at,
              }))
              .filter((s) => s.user_id !== user.id) // Exclude current user
            setActiveSessions(sessions)
          })
          .on('presence', { event: 'join' }, ({ key, newPresences }) => {
            console.log('User joined editing session:', key, newPresences)
          })
          .on('presence', { event: 'leave' }, ({ key, leftPresences }) => {
            console.log('User left editing session:', key, leftPresences)
          })
          .subscribe(async (status) => {
            if (status === 'SUBSCRIBED') {
              // Track our own presence
              await channel!.track({
                user_id: user.id,
                user_email: user.email,
                user_avatar_url: user.user_metadata?.avatar_url,
                started_at: new Date().toISOString(),
              })
            }
          })

        // Send heartbeat every 30 seconds
        heartbeatIntervalRef.current = setInterval(async () => {
          try {
            await sendHeartbeat(routeId)
          } catch (error) {
            console.error('Failed to send heartbeat:', error)
          }
        }, 30000)
      } catch (error) {
        console.error('Failed to join editing session:', error)
      }
    }

    initialize()

    // Cleanup on unmount
    return () => {
      const cleanup = async () => {
        try {
          // Clear heartbeat interval
          if (heartbeatIntervalRef.current) {
            clearInterval(heartbeatIntervalRef.current)
            heartbeatIntervalRef.current = null
          }

          // Untrack presence and unsubscribe
          if (channel) {
            await channel.untrack()
            supabase.removeChannel(channel)
          }

          // Leave editing session
          if (routeId) {
            await leaveEditingSession(routeId)
          }

          setEditingSession(null)
          setActiveSessions([])
        } catch (error) {
          console.error('Failed to cleanup editing session:', error)
        }
      }

      cleanup()
    }
  }, [routeId, user, setEditingSession, setActiveSessions])
}
