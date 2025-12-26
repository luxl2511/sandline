'use client'

import type { Route } from '@/types'

interface RouteListProps {
  routes: Route[]
  onRouteSelect: (route: Route) => void
}

export default function RouteList({ routes, onRouteSelect }: RouteListProps) {
  return (
    <div className="bg-white shadow-md rounded-lg p-4 max-w-sm">
      <h2 className="text-lg font-semibold mb-2">Routes</h2>
      {routes.length === 0 ? (
        <p className="text-gray-500">No routes found.</p>
      ) : (
        <ul className="divide-y divide-gray-200">
          {routes.map((route) => (
            <li
              key={route.id}
              className="py-2 px-2 hover:bg-gray-100 cursor-pointer rounded"
              onClick={() => onRouteSelect(route)}
            >
              <div className="flex justify-between">
                <span className="font-medium">{route.name}</span>
                <span className="text-sm text-gray-500">
                  {new Date(route.createdAt).toLocaleDateString()}
                </span>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
