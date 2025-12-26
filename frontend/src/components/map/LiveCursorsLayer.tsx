'use client'

import { Marker } from 'react-map-gl'
import type { CursorPosition } from '@/hooks/useLiveCursors'
import { useMapStore } from '@/lib/store'

interface LiveCursorsLayerProps {
  cursors: CursorPosition[]
}

/**
 * Generate consistent hex color from email hash
 */
function getColorFromEmail(email: string): string {
  let hash = 0
  for (let i = 0; i < email.length; i++) {
    hash = email.charCodeAt(i) + ((hash << 5) - hash)
  }

  const colors = [
    '#3b82f6', // blue
    '#22c55e', // green
    '#a855f7', // purple
    '#ec4899', // pink
    '#6366f1', // indigo
    '#eab308', // yellow
    '#ef4444', // red
  ]

  return colors[Math.abs(hash) % colors.length]
}

/**
 * Renders SVG cursor icons for each collaborator
 *
 * Features:
 * - Consistent color per user (from email hash)
 * - Username label next to cursor
 * - Smooth cursor movement via Marker positioning
 */
export default function LiveCursorsLayer({ cursors }: LiveCursorsLayerProps) {
  const { activeSessions } = useMapStore()

  return (
    <>
      {cursors.map((cursor) => {
        const session = activeSessions.find((s) => s.user_id === cursor.user_id)
        const color = getColorFromEmail(cursor.user_email)
        const username = cursor.user_email.split('@')[0]

        return (
          <Marker key={cursor.user_id} longitude={cursor.lng} latitude={cursor.lat}>
            <div className="relative pointer-events-none">
              {/* Cursor SVG */}
              <svg
                width="20"
                height="20"
                viewBox="0 0 20 20"
                style={{ filter: 'drop-shadow(0 1px 2px rgba(0,0,0,0.3))' }}
              >
                <path
                  d="M0 0 L0 16 L6 12 L10 20 L12 19 L8 11 L16 11 Z"
                  fill={color}
                  stroke="white"
                  strokeWidth="1"
                />
              </svg>

              {/* User label */}
              <div
                className="absolute top-4 left-4 px-2 py-1 rounded text-xs text-white whitespace-nowrap font-medium shadow-lg"
                style={{ backgroundColor: color }}
              >
                {username}
              </div>
            </div>
          </Marker>
        )
      })}
    </>
  )
}
