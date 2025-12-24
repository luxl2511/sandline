'use client'

import { useState } from 'react'
import { useAuth } from '@/contexts/AuthContext'
import AuthModal from './AuthModal'

export default function AuthButton() {
  const { user, signOut, loading } = useAuth()
  const [showAuthModal, setShowAuthModal] = useState(false)
  const [showMenu, setShowMenu] = useState(false)

  if (loading) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg px-4 py-2">
        <span className="text-sm text-gray-500 dark:text-gray-400">Loading...</span>
      </div>
    )
  }

  if (!user) {
    return (
      <>
        <button
          onClick={() => setShowAuthModal(true)}
          className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg shadow-lg text-sm font-medium"
        >
          Sign In
        </button>

        {showAuthModal && <AuthModal onClose={() => setShowAuthModal(false)} />}
      </>
    )
  }

  return (
    <>
      <div className="relative">
        <button
          onClick={() => setShowMenu(!showMenu)}
          className="bg-white dark:bg-gray-800 rounded-lg shadow-lg px-4 py-2 text-sm text-gray-900 dark:text-gray-50 hover:bg-gray-50 dark:hover:bg-gray-700"
        >
          {user.email}
        </button>

        {showMenu && (
          <>
            {/* Backdrop to close menu */}
            <div
              className="fixed inset-0 z-10"
              onClick={() => setShowMenu(false)}
            />

            {/* Dropdown menu */}
            <div className="absolute top-full right-0 mt-2 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-20 min-w-[160px]">
              <button
                onClick={async () => {
                  setShowMenu(false)
                  await signOut()
                }}
                className="w-full text-left px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-gray-50 dark:hover:bg-gray-700 rounded-lg"
              >
                Sign Out
              </button>
            </div>
          </>
        )}
      </div>
    </>
  )
}
