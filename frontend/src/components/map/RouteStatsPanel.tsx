'use client'

import React from 'react'
import type { Route } from '@/types'

interface RouteStatsPanelProps {
  route: Route // The route for which to display statistics
}

export default function RouteStatsPanel({ route }: RouteStatsPanelProps) {
  const kilometers = route.length_km ?? 0
  const estimatedTimeMin = route.estimated_time_min ?? 0
  const lastChanged = new Date(route.created_at).toLocaleString()
  const lastChangedBy = route.created_by || route.owner_id

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4 w-64 text-sm">
      <h4 className="font-bold mb-2 text-gray-900 dark:text-gray-50">Route Stats</h4>
      <p className="text-gray-700 dark:text-gray-300">Name: <span className="font-semibold">{route.name}</span></p>
      <p className="text-gray-700 dark:text-gray-300">Length: {kilometers > 0 ? kilometers.toFixed(2) : '--'} km</p>
      <p className="text-gray-700 dark:text-gray-300">Time: {estimatedTimeMin > 0 ? estimatedTimeMin : '--'} min</p>
      <p className="text-gray-700 dark:text-gray-300">Last Changed: {lastChanged}</p>
      <p className="text-gray-700 dark:text-gray-300">By: <span className="truncate inline-block max-w-[150px] align-bottom">{lastChangedBy}</span></p>
    </div>
  )
}
