import { useEffect, useRef } from "react";
import MapboxDraw from "@mapbox/mapbox-gl-draw";
import type { MapRef } from "react-map-gl";
import "@mapbox/mapbox-gl-draw/dist/mapbox-gl-draw.css";

export interface UseMapboxDrawOptions {
  enabled: boolean;
  onDrawCreate?: (features: GeoJSON.Feature[]) => void;
  onDrawUpdate?: (features: GeoJSON.Feature[]) => void;
  onDrawDelete?: (features: GeoJSON.Feature[]) => void;
}

export default function useMapboxDraw(
  mapRef: React.RefObject<MapRef>,
  options: UseMapboxDrawOptions,
) {
  const drawRef = useRef<MapboxDraw | null>(null);

  useEffect(() => {
    const map = mapRef.current?.getMap();
    if (!map || !options.enabled) return;

    // Initialize Mapbox Draw
    const draw = new MapboxDraw({
      displayControlsDefault: false,
      controls: {
        line_string: true,
        trash: true,
      },
      defaultMode: "draw_line_string",
      styles: [
        // Custom styling for draw mode
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

    // Event handlers
    map.on("draw.create", (e: { features: GeoJSON.Feature[] }) => {
      options.onDrawCreate?.(e.features);
    });
    map.on("draw.update", (e: { features: GeoJSON.Feature[] }) => {
      options.onDrawUpdate?.(e.features);
    });
    map.on("draw.delete", (e: { features: GeoJSON.Feature[] }) => {
      options.onDrawDelete?.(e.features);
    });

    return () => {
      if (draw && map.hasControl(draw)) {
        map.removeControl(draw);
      }
      drawRef.current = null;
    };
  }, [
    mapRef,
    options,
    options.enabled,
    options.onDrawCreate,
    options.onDrawUpdate,
    options.onDrawDelete,
  ]);

  return drawRef;
}
