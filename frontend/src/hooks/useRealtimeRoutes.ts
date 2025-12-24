import { useEffect } from "react";
import { supabase } from "@/lib/supabase";
import { fetchRoutes } from "@/lib/api";

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
      .subscribe();

    // Cleanup subscription on unmount
    return () => {
      supabase.removeChannel(channel);
    };
  }, [options, options.onRoutesChange]);
}
