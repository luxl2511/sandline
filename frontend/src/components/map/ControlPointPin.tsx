'use client'

import { useAuth } from '@/contexts/AuthContext'
import { useMemo } from 'react'

interface ControlPointPinProps {
  userEmail?: string | null // User dragging this point, if any
  isMine: boolean // Is this point being dragged by the current user
  onDelete: () => void
}

export default function ControlPointPin({ userEmail, isMine, onDelete }: ControlPointPinProps) {
  const { user: currentUser } = useAuth()

  const borderColorClass = useMemo(() => {
    if (isMine) return 'border-blue-500' // Current user's point
    if (userEmail) return 'border-red-500' // Another user is dragging
    return 'border-gray-400' // No one is dragging
  }, [isMine, userEmail])

  const backgroundColorClass = useMemo(() => {
    if (userEmail) return 'bg-red-200' // Another user is dragging
    return 'bg-white' // Default
  }, [userEmail])

  return (
    <div className="relative group">
      <div
        className={`relative w-6 h-6 rounded-full flex items-center justify-center
                    ${backgroundColorClass} ${borderColorClass} border-2 shadow-md
                    transition-all duration-100 ease-out transform hover:scale-125`}
      >
        {userEmail && (
          <span
            className={`absolute bottom-full mb-1 px-1.5 py-0.5 text-xs font-semibold text-white
                        rounded-full bg-red-500 whitespace-nowrap`}
          >
            {userEmail.split('@')[0]}
          </span>
        )}
        {!userEmail && currentUser && isMine && (
          <span
            className={`absolute bottom-full mb-1 px-1.5 py-0.5 text-xs font-semibold text-white
                        rounded-full bg-blue-500 whitespace-nowrap`}
          >
            Me
          </span>
        )}
        <div className="w-2 h-2 rounded-full bg-gray-700" />
      </div>
      <button
        onClick={(e) => {
          e.stopPropagation() // Prevent map click events
          onDelete()
        }}
        className="absolute top-0 right-0 -mt-2 -mr-2 w-4 h-4 bg-red-500 text-white text-xs rounded-full
                   flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity"
      >
        x
      </button>
    </div>
  )
}
