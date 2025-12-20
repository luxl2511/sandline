'use client'

import { useMapStore } from '@/lib/store'

export default function LayerControls() {
  const { layers, toggleLayer } = useMapStore()

  const layerConfig = [
    { key: 'osmTracks' as const, label: 'OSM Tracks' },
    { key: 'curatedTracks' as const, label: 'Curated Tracks' },
    { key: 'satellite' as const, label: 'Satellite' },
    { key: 'routes' as const, label: 'My Routes' },
  ]

  return (
    <div className="bg-white rounded-lg shadow-lg p-4 space-y-2">
      <h3 className="font-bold text-sm mb-3">Layers</h3>
      {layerConfig.map(({ key, label }) => (
        <label key={key} className="flex items-center space-x-2 cursor-pointer">
          <input
            type="checkbox"
            checked={layers[key]}
            onChange={() => toggleLayer(key)}
            className="w-4 h-4 rounded border-gray-300"
          />
          <span className="text-sm">{label}</span>
        </label>
      ))}

      <div className="mt-4 pt-4 border-t border-gray-200">
        <h4 className="font-semibold text-xs mb-2">Confidence</h4>
        <div className="space-y-1 text-xs">
          <div className="flex items-center space-x-2">
            <div className="w-4 h-1 bg-track-confidence-5"></div>
            <span>5 - Rally verified</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-4 h-1 bg-track-confidence-4"></div>
            <span>4 - Community</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-4 h-1 bg-track-confidence-3"></div>
            <span>3 - OSM visible</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-4 h-1 bg-track-confidence-2"></div>
            <span>2 - Satellite</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-4 h-1 bg-track-confidence-1"></div>
            <span>1 - Estimated</span>
          </div>
        </div>
      </div>
    </div>
  )
}
