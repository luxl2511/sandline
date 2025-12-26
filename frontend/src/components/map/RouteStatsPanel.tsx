'use client'

import React from 'react'
import type { Route } from '@/types'

interface RouteStatsPanelProps {
  route: Route // The route for which to display statistics
}

export default function RouteStatsPanel({ route }: RouteStatsPanelProps) {
  // Placeholder values for now
  const kilometers = 123.45
  const estimatedTimeMin = 180
  const lastChanged = new Date().toLocaleDateString()
  const lastChangedBy = "user@example.com" // Placeholder

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-4 w-64 text-sm">
      <h4 className="font-bold mb-2 text-gray-900 dark:text-gray-50">Route Stats</h4>
      <p className="text-gray-700 dark:text-gray-300">Name: <span className="font-semibold">{route.name}</span></p>
      <p className="text-gray-700 dark:text-gray-300">Owner: <span className="font-semibold">{route.ownerId}</span></p> {/* Will need to map ownerId to email */}
      <p className="text-gray-700 dark:text-gray-300">Length: {kilometers.toFixed(2)} km</p>
      <p className="text-gray-700 dark:text-gray-300">Time: {estimatedTimeMin} min</p>
      <p className="text-gray-700 dark:text-gray-300">Last Changed: {lastChanged}</p>
      <p className="text-gray-700 dark:text-gray-300">By: {lastChangedBy}</p>
    </div>
  )
}
