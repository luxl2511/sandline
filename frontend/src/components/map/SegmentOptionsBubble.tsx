'use client'

import React from 'react'
import { Popup } from 'react-map-gl'

interface SegmentOptionsBubbleProps {
  longitude: number
  latitude: number
  onClose: () => void
  // Add more props for segment info and routing options as needed
  segmentLengthKm: number
  estimatedTimeMin: number
}

export default function SegmentOptionsBubble({
  longitude,
  latitude,
  onClose,
  segmentLengthKm,
  estimatedTimeMin,
}: SegmentOptionsBubbleProps) {
  return (
    <Popup
      longitude={longitude}
      latitude={latitude}
      onClose={onClose}
      closeButton={false}
      closeOnClick={false}
      anchor="bottom"
    >
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-3 text-sm">
        <h4 className="font-bold mb-1 text-gray-900 dark:text-gray-50">Segment Info</h4>
        <p className="text-gray-700 dark:text-gray-300">Length: {segmentLengthKm.toFixed(2)} km</p>
        <p className="text-gray-700 dark:text-gray-300">Time: {estimatedTimeMin.toFixed(0)} min</p>

        <div className="mt-2 space-x-2">
          <button className="bg-blue-500 hover:bg-blue-600 text-white px-2 py-1 rounded text-xs">
            Road Routing
          </button>
          <button className="bg-gray-200 hover:bg-gray-300 text-gray-800 px-2 py-1 rounded text-xs">
            Free Routing
          </button>
        </div>
      </div>
    </Popup>
  )
}
