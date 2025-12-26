'use client'

import { useAuth } from '@/contexts/AuthContext'
import { useMemo } from 'react'

interface ControlPointPinProps {
  user_email?: string | null
  isMine: boolean
  onDelete: () => void
  type: 'start' | 'end' | 'intermediate'
}

export default function ControlPointPin({ user_email, isMine, onDelete, type }: ControlPointPinProps) {
  const { user: currentUser } = useAuth()

  // Base colors based on state
  const colorClasses = useMemo(() => {
    if (user_email) return 'bg-red-500 border-red-300 shadow-red-500/80' // Dragging by other
    if (isMine) return 'bg-yellow-400 border-yellow-200 shadow-yellow-400/80' // Active/Mine (Gold)

    // Default based on type
    switch (type) {
      case 'start': return 'bg-emerald-500 border-emerald-300 shadow-emerald-500/60'
      case 'end': return 'bg-rose-600 border-rose-300 shadow-rose-600/60'
      case 'intermediate': return 'bg-blue-500 border-blue-300 shadow-blue-500/60'
    }
  }, [isMine, user_email, type])

  // Shape classes based on type
  const shapeClasses = useMemo(() => {
    switch (type) {
      case 'start': return 'clip-path-triangle translate-y-[-2px]' // Custom class or style for triangle
      case 'end': return 'rounded-sm' // Square
      case 'intermediate': return 'rotate-45 rounded-sm scale-75' // Diamond
    }
  }, [type])

  return (
    <div className="relative group cursor-pointer" style={{ isolation: 'isolate' }}>
      {/* Floating Label (User or 'You') */}
      {(user_email || (currentUser && isMine)) && (
        <div className={`
          absolute -top-10 left-1/2 -translate-x-1/2
          px-2 py-1 text-[10px] font-bold text-white uppercase tracking-wider
          rounded bg-gray-900/90 border border-white/20 backdrop-blur-sm
          shadow-lg transform transition-all duration-200
          ${user_email ? 'scale-110 z-20' : 'scale-100 z-10'}
        `}>
          {user_email ? user_email.split('@')[0] : 'YOU'}
        </div>
      )}

      {/* Main Pin Container - Handles Hover Scale & bounce */}
      <div className={`
        relative flex items-center justify-center
        w-8 h-8
        transition-transform duration-200 cubic-bezier(0.175, 0.885, 0.32, 1.275)
        group-hover:scale-125
        ${user_email ? 'scale-125' : ''}
      `}>

        {/* Glow/Shadow (Behind) */}
        <div className={`
          absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2
          w-10 h-10 rounded-full blur-md opacity-60
          ${colorClasses.split(' ').find(c => c.startsWith('bg-'))?.replace('bg-', 'bg-')}
        `} />

        {/* The Geometric Shape */}
        <div
          className={`
            w-6 h-6
            border-[3px]
            ${colorClasses}
            ${shapeClasses}
            flex items-center justify-center
            relative z-10
            transition-colors duration-200
          `}
          style={type === 'start' ? { clipPath: 'polygon(100% 50%, 0 0, 0 100%)' } : {}}
        >
          {/* Inner details (e.g. dot or hole) */}
          {type !== 'start' && (
            <div className={`
              w-1.5 h-1.5 bg-white rounded-full shadow-inner
              ${type === 'intermediate' ? '-rotate-45' : ''}
            `} />
          )}
        </div>

        {/* Pulse Ring for Active Drag */}
        {user_email && (
          <div className="absolute inset-0 rounded-full border-2 border-red-500 animate-ping opacity-50" />
        )}
      </div>

      {/* Delete Button (Appears on Hover) */}
      <button
        onClick={(e) => {
          e.stopPropagation()
          onDelete()
        }}
        className={`
          absolute -top-3 -right-3
          w-5 h-5
          bg-red-500 text-white text-xs font-bold
          rounded flex items-center justify-center
          shadow-md border border-white/50
          opacity-0 group-hover:opacity-100
          transform scale-75 group-hover:scale-100
          transition-all duration-200
          z-20
        `}
      >
        ×
      </button>
    </div>
  )
}
