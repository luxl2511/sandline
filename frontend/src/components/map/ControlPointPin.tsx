'use client'

import { useAuth } from '@/contexts/AuthContext'
import { useMemo } from 'react'

interface ControlPointPinProps {
  user_email?: string | null // User dragging this point, if any
  isMine: boolean // Is this point being dragged by the current user
  onDelete: () => void
}

export default function ControlPointPin({ user_email, isMine, onDelete }: ControlPointPinProps) {
  const { user: currentUser } = useAuth()

  const pinColor = useMemo(() => {
    if (isMine) return 'from-blue-500 to-blue-600' // Current user's point
    if (user_email) return 'from-red-500 to-red-600' // Another user is dragging
    return 'from-purple-500 to-purple-600' // Default - gamey purple
  }, [isMine, user_email])

  const glowColor = useMemo(() => {
    if (isMine) return 'shadow-blue-500/50' // Current user's point
    if (user_email) return 'shadow-red-500/50' // Another user is dragging
    return 'shadow-purple-500/50' // Default
  }, [isMine, user_email])

  return (
    <div className="relative group">
      {/* Map Pin Shape */}
      <div className={`
        relative flex flex-col items-center
        transition-all duration-200 ease-out transform hover:scale-110
        ${user_email ? 'animate-pulse' : ''}
      `}>
        {/* Pin Head - Teardrop/Pin shape */}
        <div className={`
          w-10 h-10 rounded-full
          bg-gradient-to-br ${pinColor}
          border-2 border-white
          shadow-lg ${glowColor}
          flex items-center justify-center
          relative
        `}>
          {/* Inner dot */}
          <div className="w-3 h-3 rounded-full bg-white/90" />

          {/* Pulse animation ring for active drag */}
          {user_email && (
            <div className="absolute inset-0 rounded-full bg-red-400/30 animate-ping" />
          )}
        </div>

        {/* Pin Point/Tail */}
        <div className={`
          w-0 h-0
          border-l-[8px] border-l-transparent
          border-r-[8px] border-r-transparent
          border-t-[12px] ${isMine ? 'border-t-blue-600' : user_email ? 'border-t-red-600' : 'border-t-purple-600'}
          filter drop-shadow-md
          -mt-1
        `} />

        {/* User label */}
        {user_email && (
          <span className={`
            absolute -top-8 left-1/2 -translate-x-1/2
            px-2 py-1 text-xs font-bold text-white
            rounded-md bg-red-600 whitespace-nowrap
            shadow-lg animate-bounce
          `}>
            {user_email.split('@')[0]}
          </span>
        )}

        {!user_email && currentUser && isMine && (
          <span className={`
            absolute -top-8 left-1/2 -translate-x-1/2
            px-2 py-1 text-xs font-bold text-white
            rounded-md bg-blue-600 whitespace-nowrap
            shadow-lg
          `}>
            You
          </span>
        )}
      </div>

      {/* Delete button */}
      <button
        onClick={(e) => {
          e.stopPropagation() // Prevent map click events
          onDelete()
        }}
        className={`
          absolute -top-1 -right-1 w-6 h-6
          bg-gradient-to-br from-red-500 to-red-600
          text-white text-sm font-bold rounded-full
          flex items-center justify-center
          border-2 border-white
          opacity-0 group-hover:opacity-100
          transition-all duration-200
          hover:scale-110
          shadow-lg
          z-10
        `}
      >
        ×
      </button>
    </div>
  )
}
