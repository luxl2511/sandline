import { useEffect, useRef, useState } from "react";
import MapboxDraw from "@mapbox/mapbox-gl-draw";
import type { MapRef } from "react-map-gl";
import "@mapbox/mapbox-gl-draw/dist/mapbox-gl-draw.css";
import { createPointChange } from "@/lib/api";
import { useRealtimeBroadcast } from "./useRealtimeBroadcast";

export interface UseCollaborativeMapboxDrawOptions {
  routeId: string | null;
  currentGeometry: GeoJSON.MultiLineString | null;
  onPointMoved?: (change: {
    featureIndex: number;
    pointIndex: number;
    originalPosition: [number, number];
    newPosition: [number, number];
  }) => void;
}

const EPSILON = 0.000001; // ~11cm at equator

/**
 * Find which point moved by comparing old and new geometries
 */
function findMovedPoint(
  oldGeom: GeoJSON.MultiLineString,
  newGeom: GeoJSON.MultiLineString
): {
  featureIndex: number;
  pointIndex: number;
  originalPosition: [number, number];
  newPosition: [number, number];
} | null {
  for (let fIdx = 0; fIdx < oldGeom.coordinates.length; fIdx++) {
    const oldCoords = oldGeom.coordinates[fIdx];
    const newCoords = newGeom.coordinates[fIdx];

    if (oldCoords.length !== newCoords.length) {
      // Point added/removed - different operation, not a move
      return null;
    }

    for (let pIdx = 0; pIdx < oldCoords.length; pIdx++) {
      const [oldLng, oldLat] = oldCoords[pIdx];
      const [newLng, newLat] = newCoords[pIdx];

      if (
        Math.abs(oldLng - newLng) > EPSILON ||
        Math.abs(oldLat - newLat) > EPSILON
      ) {
        return {
          featureIndex: fIdx,
          pointIndex: pIdx,
          originalPosition: [oldLng, oldLat],
          newPosition: [newLng, newLat],
        };
      }
    }
  }

  return null;
}

export default function useCollaborativeMapboxDraw(
  mapRef: React.RefObject<MapRef>,
  options: UseCollaborativeMapboxDrawOptions
) {
  const { broadcast } = useRealtimeBroadcast(options.routeId);
  const drawRef = useRef<MapboxDraw | null>(null);
  const originalGeometryRef = useRef<GeoJSON.MultiLineString | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStartPoint, setDragStartPoint] = useState<{
    featureIndex: number;
    pointIndex: number;
    originalPosition: [number, number];
  } | null>(null);

  useEffect(() => {
    const map = mapRef.current?.getMap();
    if (!map || !options.routeId || !options.currentGeometry) {
      // Remove draw control if disabled
      if (drawRef.current && map) {
        map.removeControl(drawRef.current);
        drawRef.current = null;
      }
      return;
    }

    // Store original geometry
    originalGeometryRef.current = options.currentGeometry;

    // Initialize Mapbox Draw with edit mode
    const draw = new MapboxDraw({
      displayControlsDefault: false,
      controls: {
        trash: false, // Disable deletion during collaborative editing
      },
      defaultMode: "simple_select",
      styles: [
        // Custom styling for edit mode
        {
          id: "gl-draw-line",
          type: "line",
          filter: [
            "all",
            ["==", "$type", "LineString"],
            ["!=", "mode", "static"],
          ],
          layout: {
            "line-cap": "round",
            "line-join": "round",
          },
          paint: {
            "line-color": "#3b82f6", // blue-500
            "line-width": 3,
          },
        },
        {
          id: "gl-draw-polygon-and-line-vertex-halo-active",
          type: "circle",
          filter: ["all", ["==", "meta", "vertex"], ["==", "$type", "Point"]],
          paint: {
            "circle-radius": 7,
            "circle-color": "#fff",
          },
        },
        {
          id: "gl-draw-polygon-and-line-vertex-active",
          type: "circle",
          filter: ["all", ["==", "meta", "vertex"], ["==", "$type", "Point"]],
          paint: {
            "circle-radius": 5,
            "circle-color": "#3b82f6",
          },
        },
      ],
    });

    map.addControl(draw, "top-left");
    drawRef.current = draw;

    // Load current route geometry into draw control
    const feature: GeoJSON.Feature<GeoJSON.MultiLineString> = {
      type: "Feature",
      properties: {},
      geometry: options.currentGeometry,
    };
    draw.add(feature);

    // Enter direct_select mode to allow vertex dragging
    const featureIds = draw.getAll().features.map((f) => f.id as string);
    if (featureIds.length > 0) {
      draw.changeMode("direct_select", { featureId: featureIds[0] });
    }

    // Event handler for drag start (selection change)
    const handleSelectionChange = () => {
      // Reset dragging state when selection changes
      setIsDragging(false);
      setDragStartPoint(null);
    };

    // Event handler for vertex updates (throttled broadcast)
    let throttleTimer: NodeJS.Timeout | null = null;
    const handleUpdate = (e: { features: GeoJSON.Feature[] }) => {
      if (e.features.length === 0 || !originalGeometryRef.current) return;

      const updatedFeature = e.features[0];
      if (updatedFeature.geometry.type !== "MultiLineString") return;

      const movedPoint = findMovedPoint(
        originalGeometryRef.current,
        updatedFeature.geometry as GeoJSON.MultiLineString
      );

      if (movedPoint && !isDragging) {
        // Drag just started
        setIsDragging(true);
        setDragStartPoint({
          featureIndex: movedPoint.featureIndex,
          pointIndex: movedPoint.pointIndex,
          originalPosition: movedPoint.originalPosition,
        });

        broadcast("drag_start", {
          feature_index: movedPoint.featureIndex,
          point_index: movedPoint.pointIndex,
          original_position: movedPoint.originalPosition,
        });
      } else if (movedPoint && isDragging) {
        // Drag update - throttled to 20fps (50ms)
        if (!throttleTimer) {
          throttleTimer = setTimeout(() => {
            broadcast("drag_update", {
              feature_index: movedPoint.featureIndex,
              point_index: movedPoint.pointIndex,
              new_position: movedPoint.newPosition,
            });
            throttleTimer = null;
          }, 50);
        }
      }
    };

    // Event handler for mode change (drag end)
    const handleModeChange = async (e: { mode: string }) => {
      if (!isDragging || !dragStartPoint) return;

      // Drag ended - get final position
      const features = draw.getAll().features;
      if (features.length === 0) return;

      const updatedFeature = features[0];
      if (updatedFeature.geometry.type !== "MultiLineString") return;

      const movedPoint = findMovedPoint(
        originalGeometryRef.current!,
        updatedFeature.geometry as GeoJSON.MultiLineString
      );

      if (movedPoint) {
        console.log("Drag ended:", movedPoint);

        // Broadcast drag end
        broadcast("drag_end", {
          feature_index: movedPoint.featureIndex,
          point_index: movedPoint.pointIndex,
          new_position: movedPoint.newPosition,
        });

        try {
          // Submit point change to backend (authoritative processing)
          await createPointChange(options.routeId!, {
            featureIndex: movedPoint.featureIndex,
            pointIndex: movedPoint.pointIndex,
            originalPosition: movedPoint.originalPosition,
            newPosition: movedPoint.newPosition,
          });

          // Notify parent component if callback provided
          options.onPointMoved?.(movedPoint);

          // NOW revert (after backend receives it)
          // Keep original position until change is accepted by route owner
          draw.deleteAll();
          draw.add(feature);
          if (featureIds.length > 0) {
            draw.changeMode("direct_select", { featureId: featureIds[0] });
          }
        } catch (error) {
          console.error("Failed to create point change:", error);
          // Revert on error
          draw.deleteAll();
          draw.add(feature);
          if (featureIds.length > 0) {
            draw.changeMode("direct_select", { featureId: featureIds[0] });
          }
        }
      }

      // Reset dragging state
      setIsDragging(false);
      setDragStartPoint(null);
    };

    map.on("draw.selectionchange", handleSelectionChange);
    map.on("draw.update", handleUpdate);
    map.on("draw.modechange", handleModeChange);

    return () => {
      map.off("draw.selectionchange", handleSelectionChange);
      map.off("draw.update", handleUpdate);
      map.off("draw.modechange", handleModeChange);
      if (draw && map.hasControl(draw)) {
        map.removeControl(draw);
      }
      if (throttleTimer) {
        clearTimeout(throttleTimer);
      }
      drawRef.current = null;
      originalGeometryRef.current = null;
    };
  }, [
    mapRef,
    options.routeId,
    options.currentGeometry,
    options.onPointMoved,
  ]);

  return drawRef;
}
