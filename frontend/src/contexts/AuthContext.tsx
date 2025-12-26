'use client'

import { createContext, useContext, useEffect, useState } from 'react'
import { supabase } from '@/lib/supabase'
import type { User, Session } from '@supabase/supabase-js'

interface AuthContextType {
  user: User | null
  session: Session | null
  loading: boolean
  signIn: (email: string, password: string) => Promise<void>
  signUp: (email: string, password: string) => Promise<void>
  signOut: () => Promise<void>
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

// LocalStorage keys for JWT caching
const JWT_STORAGE_KEY = 'dakar_planner_jwt'
const JWT_EXPIRY_KEY = 'dakar_planner_jwt_expiry'

// JWT helper functions
const storeJWT = (token: string, expiresAt: number) => {
  if (typeof window === 'undefined') return
  localStorage.setItem(JWT_STORAGE_KEY, token)
  localStorage.setItem(JWT_EXPIRY_KEY, expiresAt.toString())
}

const getCachedJWT = (): string | null => {
  if (typeof window === 'undefined') return null

  const token = localStorage.getItem(JWT_STORAGE_KEY)
  const expiry = localStorage.getItem(JWT_EXPIRY_KEY)

  if (!token || !expiry) return null

  const expiresAt = parseInt(expiry, 10)
  const now = Math.floor(Date.now() / 1000) // Current time in seconds

  // Check if token is expired (with 60s buffer to account for clock skew)
  if (now >= expiresAt - 60) {
    // Token expired or about to expire - clear it
    localStorage.removeItem(JWT_STORAGE_KEY)
    localStorage.removeItem(JWT_EXPIRY_KEY)
    return null
  }

  return token
}

const clearJWT = () => {
  if (typeof window === 'undefined') return
  localStorage.removeItem(JWT_STORAGE_KEY)
  localStorage.removeItem(JWT_EXPIRY_KEY)
}

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [session, setSession] = useState<Session | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    // Try to get cached JWT first (avoids API call)
    const cachedToken = getCachedJWT()

    if (cachedToken) {
      try {
        // Decode JWT to get user info (payload is base64url encoded)
        // NOTE: We don't validate the signature here - backend will do that
        // This is purely for UX (showing user info faster on page load)
        const payloadBase64 = cachedToken.split('.')[1]
        const payloadJson = atob(payloadBase64.replace(/-/g, '+').replace(/_/g, '/'))
        const payload = JSON.parse(payloadJson)

        // Create a minimal user object from JWT claims
        const userFromToken: User = {
          id: payload.sub,
          email: payload.email,
          app_metadata: {},
          user_metadata: {},
          aud: 'authenticated',
          created_at: '',
        }

        // Create a minimal session object
        const sessionFromToken: Session = {
          access_token: cachedToken,
          token_type: 'bearer',
          expires_in: payload.exp - Math.floor(Date.now() / 1000),
          expires_at: payload.exp,
          refresh_token: '',
          user: userFromToken,
        }

        setUser(userFromToken)
        setSession(sessionFromToken)
        setLoading(false)

        // Note: We don't call Supabase here to validate the token
        // Backend will validate it on every API request
        // This is purely client-side caching for faster page loads
        return
      } catch (error) {
        // If JWT decode fails, clear cache and fetch from Supabase
        console.warn('Failed to decode cached JWT:', error)
        clearJWT()
      }
    }

    // No cached token or decode failed - fetch from Supabase
    supabase.auth.getSession().then(({ data: { session } }) => {
      if (session?.access_token && session?.expires_at) {
        // Store JWT in localStorage for future page loads
        storeJWT(session.access_token, session.expires_at)
      }
      setSession(session)
      setUser(session?.user ?? null)
      setLoading(false)
    })

    // Listen for auth changes (sign in, sign out, token refresh)
    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((_event, session) => {
      if (session?.access_token && session?.expires_at) {
        // Update localStorage when session changes
        storeJWT(session.access_token, session.expires_at)
      } else {
        // Clear localStorage on sign out
        clearJWT()
      }
      setSession(session)
      setUser(session?.user ?? null)
      setLoading(false)
    })

    return () => subscription.unsubscribe()
  }, [])

  const signIn = async (email: string, password: string) => {
    const { error } = await supabase.auth.signInWithPassword({
      email,
      password,
    })
    if (error) throw error
    // Session will be updated via onAuthStateChange listener
  }

  const signUp = async (email: string, password: string) => {
    const { error } = await supabase.auth.signUp({
      email,
      password,
    })
    if (error) throw error
    // Session will be updated via onAuthStateChange listener
  }

  const signOut = async () => {
    // Clear localStorage first
    clearJWT()

    const { error } = await supabase.auth.signOut()
    if (error) throw error
    // Session will be updated via onAuthStateChange listener
  }

  const value = {
    user,
    session,
    loading,
    signIn,
    signUp,
    signOut,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}
