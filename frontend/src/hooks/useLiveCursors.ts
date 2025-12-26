"use client";

import { useEffect, useState, useRef } from "react";
import type mapboxgl from "mapbox-gl";
import type { MapRef } from "react-map-gl";
import { useRealtimeBroadcast } from "./useRealtimeBroadcast";

export interface CursorPosition {
  userId: string;
  userEmail: string;
  lng: number;
  lat: number;
  timestamp: number;
}

export function useLiveCursors(
  mapRef: React.RefObject<MapRef | null>,
  routeId: string | null,
) {
  const { broadcast, messages } = useRealtimeBroadcast(routeId);
  const [cursors, setCursors] = useState<Map<string, CursorPosition>>(
    new Map(),
  );
  const throttleRef = useRef<number>(0);

  // Broadcast cursor position (10fps)
  useEffect(() => {
    if (!routeId) return;

    const map = mapRef.current?.getMap();
    if (!map) return;

    const handleMouseMove = (e: mapboxgl.MapMouseEvent) => {
      const now = Date.now();
      if (now - throttleRef.current < 100) return;

      throttleRef.current = now;
      broadcast("cursor_move", {
        lng: e.lngLat.lng,
        lat: e.lngLat.lat,
      });
    };

    map.on("mousemove", handleMouseMove);

    return () => {
      map.off("mousemove", handleMouseMove);
    };
  }, [routeId, mapRef, broadcast]);

  // Process incoming cursor messages
  useEffect(() => {
    messages
      .filter((m) => m.type === "cursor_move")
      .forEach((msg) => {
        setCursors((prev) => {
          const next = new Map(prev);
          next.set(msg.userId, {
            userId: msg.userId,
            userEmail: msg.userEmail,
            lng: msg.data.lng,
            lat: msg.data.lat,
            timestamp: Date.now(),
          });
          return next;
        });
      });
  }, [messages]);

  // Cleanup stale cursors
  useEffect(() => {
    const interval = setInterval(() => {
      setCursors((prev) => {
        const now = Date.now();
        const next = new Map(prev);

        for (const [userId, cursor] of next) {
          if (now - cursor.timestamp > 5000) {
            next.delete(userId);
          }
        }

        return next.size === prev.size ? prev : next;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return { cursors: Array.from(cursors.values()) };
}
