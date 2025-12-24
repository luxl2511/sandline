import { useEffect, useRef } from "react";
import MapboxDraw from "@mapbox/mapbox-gl-draw";
import type { MapRef } from "react-map-gl";
import "@mapbox/mapbox-gl-draw/dist/mapbox-gl-draw.css";
import { createPointChange } from "@/lib/api";

export interface UseCollaborativeMapboxDrawOptions {
  enabled: boolean;
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
  const drawRef = useRef<MapboxDraw | null>(null);
  const originalGeometryRef = useRef<GeoJSON.MultiLineString | null>(null);

  useEffect(() => {
    const map = mapRef.current?.getMap();
    if (!map || !options.enabled || !options.currentGeometry || !options.routeId) {
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

    // Event handler for vertex updates
    const handleUpdate = async (e: { features: GeoJSON.Feature[] }) => {
      if (e.features.length === 0 || !originalGeometryRef.current) return;

      const updatedFeature = e.features[0];
      if (updatedFeature.geometry.type !== "MultiLineString") return;

      const movedPoint = findMovedPoint(
        originalGeometryRef.current,
        updatedFeature.geometry as GeoJSON.MultiLineString
      );

      if (movedPoint) {
        console.log("Point moved:", movedPoint);

        try {
          // Submit point change to API
          await createPointChange(options.routeId!, {
            featureIndex: movedPoint.featureIndex,
            pointIndex: movedPoint.pointIndex,
            originalPosition: movedPoint.originalPosition,
            newPosition: movedPoint.newPosition,
          });

          // Notify parent component if callback provided
          options.onPointMoved?.(movedPoint);

          // IMPORTANT: Revert the local change
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
    };

    map.on("draw.update", handleUpdate);

    return () => {
      map.off("draw.update", handleUpdate);
      if (draw && map.hasControl(draw)) {
        map.removeControl(draw);
      }
      drawRef.current = null;
      originalGeometryRef.current = null;
    };
  }, [
    mapRef,
    options.enabled,
    options.currentGeometry,
    options.routeId,
    options.onPointMoved,
  ]);

  return drawRef;
}
