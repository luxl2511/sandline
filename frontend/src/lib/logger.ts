/**
 * Client-side structured logging service
 * Logs to console and optionally to Supabase in development mode
 */

import { supabase } from './supabase'

type LogLevel = 'info' | 'warn' | 'error'

interface LogEntry {
  level: LogLevel
  message: string
  context?: Record<string, any>
  timestamp: string
  url: string
  userAgent: string
}

class Logger {
  private isDev = process.env.NODE_ENV === 'development'

  private async sendToSupabase(entry: LogEntry) {
    if (!this.isDev) return // Only log to Supabase in dev mode

    try {
      // Store logs in a 'client_logs' table (create if needed)
      await supabase.from('client_logs').insert({
        level: entry.level,
        message: entry.message,
        context: entry.context || {},
        timestamp: entry.timestamp,
        url: entry.url,
        user_agent: entry.userAgent,
      })
    } catch (error) {
      // Fail silently - don't let logging errors break the app
      console.error('[Logger] Failed to send log to Supabase:', error)
    }
  }

  private createLogEntry(level: LogLevel, message: string, context?: Record<string, any>): LogEntry {
    return {
      level,
      message,
      context,
      timestamp: new Date().toISOString(),
      url: typeof window !== 'undefined' ? window.location.href : '',
      userAgent: typeof window !== 'undefined' ? window.navigator.userAgent : '',
    }
  }

  info(message: string, context?: Record<string, any>) {
    const entry = this.createLogEntry('info', message, context)
    console.log(`[INFO] ${message}`, context || '')
    this.sendToSupabase(entry)
  }

  warn(message: string, context?: Record<string, any>) {
    const entry = this.createLogEntry('warn', message, context)
    console.warn(`[WARN] ${message}`, context || '')
    this.sendToSupabase(entry)
  }

  error(message: string, context?: Record<string, any>) {
    const entry = this.createLogEntry('error', message, context)
    console.error(`[ERROR] ${message}`, context || '')
    this.sendToSupabase(entry)
  }

  // Special methods for critical monitoring points
  mapboxError(error: any, context?: Record<string, any>) {
    this.error('Mapbox error', { ...context, error: String(error) })
  }

  supabaseConnectionError(error: any, context?: Record<string, any>) {
    this.error('Supabase connection error', { ...context, error: String(error) })
  }

  apiError(url: string, method: string, status: number, message: string) {
    this.error('API call failed', { url, method, status, message })
  }

  realtimeDisconnect(reason: string) {
    this.warn('Real-time connection disconnected', { reason })
  }

  jwtValidationFailed(error: any) {
    this.error('JWT validation failed', { error: String(error) })
  }
}

export const logger = new Logger()
