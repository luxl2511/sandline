import { useEffect } from "react";
import toast from 'react-hot-toast';
import { supabase } from "@/lib/supabase";
import { fetchRoutes } from "@/lib/api";
import { logger } from '@/lib/logger';

interface UseRealtimeRoutesOptions {
  onRoutesChange: () => void | Promise<void>;
}

export default function useRealtimeRoutes(options: UseRealtimeRoutesOptions) {
  useEffect(() => {
    // Subscribe to routes table changes
    const channel = supabase
      .channel("routes-changes")
      .on(
        "postgres_changes",
        {
          event: "*", // Listen to all events (INSERT, UPDATE, DELETE)
          schema: "public",
          table: "routes",
        },
        (payload) => {
          console.log("Routes change detected:", payload);
          options.onRoutesChange();
        },
      )
      .subscribe((status) => {
        if (status === 'SUBSCRIBED') {
          logger.info('Real-time routes subscription active')
        } else if (status === 'CHANNEL_ERROR') {
          logger.realtimeDisconnect('CHANNEL_ERROR')
          toast.error('Real-time updates disconnected', {
            id: 'realtime-routes-error',
          })
        } else if (status === 'TIMED_OUT') {
          logger.realtimeDisconnect('TIMED_OUT')
          toast.error('Connection timeout. Reconnecting...', {
            id: 'realtime-routes-timeout',
          })
        }
      });

    // Cleanup subscription on unmount
    return () => {
      supabase.removeChannel(channel);
    };
  }, [options, options.onRoutesChange]);
}
