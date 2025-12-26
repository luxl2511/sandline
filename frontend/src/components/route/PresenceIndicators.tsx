'use client'

import type { EditingSession } from '@/types'

interface PresenceIndicatorsProps {
  sessions: EditingSession[]
}

export default function PresenceIndicators({ sessions }: PresenceIndicatorsProps) {
  if (sessions.length === 0) return null

  // Show max 5 avatars, then "+N more"
  const visibleSessions = sessions.slice(0, 5)
  const remainingCount = sessions.length - visibleSessions.length

  const getInitials = (email: string) => {
    const parts = email.split('@')[0].split('.')
    if (parts.length > 1) {
      return (parts[0][0] + parts[1][0]).toUpperCase()
    }
    return email.substring(0, 2).toUpperCase()
  }

  const getColorFromEmail = (email: string) => {
    // Generate consistent color from email hash
    let hash = 0
    for (let i = 0; i < email.length; i++) {
      hash = email.charCodeAt(i) + ((hash << 5) - hash)
    }
    const colors = [
      'bg-blue-500',
      'bg-green-500',
      'bg-purple-500',
      'bg-pink-500',
      'bg-indigo-500',
      'bg-yellow-500',
      'bg-red-500',
    ]
    return colors[Math.abs(hash) % colors.length]
  }

  return (
    <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-3">
      <div className="flex items-center gap-2">
        <div className="flex -space-x-2">
          {visibleSessions.map((session) => (
            <div
              key={session.user_id}
              className={`w-8 h-8 rounded-full ${getColorFromEmail(session.user_email)} flex items-center justify-center text-white text-xs font-semibold border-2 border-white dark:border-gray-800`}
              title={session.user_email}
            >
              {session.user_avatar_url ? (
                <img
                  src={session.user_avatar_url}
                  alt={session.user_email}
                  className="w-full h-full rounded-full"
                />
              ) : (
                getInitials(session.user_email)
              )}
            </div>
          ))}
          {remainingCount > 0 && (
            <div className="w-8 h-8 rounded-full bg-gray-400 dark:bg-gray-600 flex items-center justify-center text-white text-xs font-semibold border-2 border-white dark:border-gray-800">
              +{remainingCount}
            </div>
          )}
        </div>
        <div className="flex-1">
          <p className="text-sm font-medium text-blue-900 dark:text-blue-100">
            {sessions.length === 1
              ? '1 person editing'
              : `${sessions.length} people editing`}
          </p>
        </div>
      </div>
      {sessions.length > 0 && (
        <div className="mt-2 text-xs text-blue-700 dark:text-blue-300">
          {sessions.map((s) => s.user_email.split('@')[0]).join(', ')}
        </div>
      )}
    </div>
  )
}
