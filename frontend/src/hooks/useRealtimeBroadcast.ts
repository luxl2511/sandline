'use client'

import { useEffect, useState, useCallback } from 'react'
import type { RealtimeChannel } from '@supabase/supabase-js'
import { supabase } from '@/lib/supabase'
import { useAuth } from '@/contexts/AuthContext'

interface BroadcastMessage {
  type: 'cursor_move' | 'drag_start' | 'drag_update' | 'drag_end'
  user_id: string
  user_email: string
  data: any
}

/**
 * Generic hook for sending/receiving broadcast messages on a route channel
 *
 * Provides ephemeral real-time messaging for collaborative editing features:
 * - Live cursor positions
 * - Ghost route drags
 * - Instant visual feedback
 *
 * Messages are NOT persisted and filtered to exclude own messages
 */
export function useRealtimeBroadcast(routeId: string | null) {
  const { user } = useAuth()
  const [messages, setMessages] = useState<BroadcastMessage[]>([])
  const [channel, setChannel] = useState<RealtimeChannel | null>(null)

  useEffect(() => {
    if (!routeId || !user) {
      if (channel) {
        channel.unsubscribe()
        setChannel(null)
      }
      return
    }

    const newChannel = supabase.channel(`route-broadcast:${routeId}`)

    // Listen to broadcast events
    newChannel
      .on('broadcast', { event: 'ephemeral' }, (payload) => {
        if (payload.payload.user_id !== user.id) {
          // Only process messages from other users (no echo)
          setMessages((prev) => [...prev, payload.payload])
        }
      })
      .subscribe()

    setChannel(newChannel)

    return () => {
      newChannel.unsubscribe()
      setChannel(null)
    }
  }, [routeId, user])

  const broadcast = useCallback(
    (type: BroadcastMessage['type'], data: any) => {
      if (!channel || !user) return

      channel.send({
        type: 'broadcast',
        event: 'ephemeral',
        payload: {
          type,
          user_id: user.id,
          user_email: user.email,
          data,
        },
      })
    },
    [channel, user]
  )

  return { messages, broadcast }
}
