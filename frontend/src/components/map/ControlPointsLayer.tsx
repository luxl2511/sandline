'use client'

import { Marker } from 'react-map-gl'
import { useEffect, useState, useMemo, useCallback } from 'react'
import type { Route } from '@/types'
import { updateRouteControlPoints } from '@/lib/api'
import { useAuth } from '@/contexts/AuthContext'
import { useRealtimeBroadcast } from '@/hooks/useRealtimeBroadcast'
import ControlPointPin from './ControlPointPin'
import { useMapStore } from '@/lib/store'

interface ControlPointsLayerProps {
  routes: Route[]
}

interface ActiveDrag {
  user_id: string
  user_email: string
  route_id: string
  point_index: number
}

export default function ControlPointsLayer({ routes }: ControlPointsLayerProps) {
  const { user } = useAuth()
  const { broadcast, messages } = useRealtimeBroadcast(null) // Listen to all broadcasts
  const [activeDrags, setActiveDrags] = useState<Map<string, ActiveDrag>>(new Map())
  const { startDrawing, setDrawnGeometry } = useMapStore()

  // Process incoming broadcast messages for drag events
  useEffect(() => {
    messages.forEach(msg => {
      if (msg.type === 'drag_start' || msg.type === 'drag_update') {
        const key = `${msg.data.route_id}-${msg.data.point_index}`
        setActiveDrags(prev => {
          const next = new Map(prev)
          next.set(key, {
            user_id: msg.user_id,
            user_email: msg.user_email,
            route_id: msg.data.route_id,
            point_index: msg.data.point_index,
          })
          return next
        })
      } else if (msg.type === 'drag_end') {
        const key = `${msg.data.route_id}-${msg.data.point_index}`
        setActiveDrags(prev => {
          const next = new Map(prev)
          next.delete(key)
          return next
        })
      }
    })
  }, [messages])

  const onDragStart = (routeId: string, pointIndex: number) => {
    if (!user) return
    const key = `${routeId}-${pointIndex}`
    setActiveDrags(prev => {
      const next = new Map(prev)
      next.set(key, { user_id: user.id, user_email: user.email || '', route_id: routeId, point_index: pointIndex })
      return next
    })
    broadcast('drag_start', { route_id: routeId, point_index: pointIndex })
  }

  const onDrag = (e: any, routeId: string, pointIndex: number) => {
    if (!user) return
    broadcast('drag_update', {
      route_id: routeId,
      point_index: pointIndex,
      new_position: [e.lngLat.lng, e.lngLat.lat],
    })
  }

  const onDragEnd = async (e: any, routeId: string, pointIndex: number) => {
    if (!user) return
    const key = `${routeId}-${pointIndex}`
    setActiveDrags(prev => {
      const next = new Map(prev)
      next.delete(key)
      return next
    })
    broadcast('drag_end', {
      route_id: routeId,
      point_index: pointIndex,
      new_position: [e.lngLat.lng, e.lngLat.lat],
    })

    const route = routes.find(r => r.id === routeId)
    if (!route || !route.control_points) return

    const newControlPoints = [...route.control_points]
    newControlPoints[pointIndex] = {
      type: 'Point',
      coordinates: [e.lngLat.lng, e.lngLat.lat],
    }

    try {
      await updateRouteControlPoints(routeId, {
        controlPoints: newControlPoints,
        featureIndex: 0, // Assuming a single feature for now
        pointIndex,
      })
    } catch (error) {
      console.error('Failed to update control points:', error)
    }
  }

  const handleExtendRoute = useCallback((point: GeoJSON.Point) => {
    startDrawing()
    setDrawnGeometry([{ type: 'Feature', geometry: point, properties: {} }])
  }, [startDrawing, setDrawnGeometry])

  const onDelete = async (routeId: string, pointIndex: number) => {
    const route = routes.find(r => r.id === routeId)
    if (!route || !route.control_points) return

    const newControlPoints = [...route.control_points]
    newControlPoints.splice(pointIndex, 1)

    try {
      await updateRouteControlPoints(routeId, {
        controlPoints: newControlPoints,
        featureIndex: 0, // Assuming a single feature for now
        pointIndex,
      })
    } catch (error) {
      console.error('Failed to update control points:', error)
    }
  }

  return (
    <>
      {routes.map(route =>
        route.control_points?.map((point, index) => {
          const key = `${route.id}-${index}`
          const activeDrag = activeDrags.get(key)
          const isBeingDraggedByMe = activeDrag?.user_id === user?.id
          const isBeingDraggedByOther = activeDrag && activeDrag.user_id !== user?.id
          const isStartOrEnd = index === 0 || index === (route.control_points?.length || 0) - 1

          return (
            <Marker
              key={key}
              longitude={point.coordinates[0]}
              latitude={point.coordinates[1]}
              draggable={!isBeingDraggedByOther} // Only draggable if not being dragged by another user
              onDragStart={() => onDragStart(route.id, index)}
              onDrag={(e) => onDrag(e, route.id, index)}
              onDragEnd={(e) => onDragEnd(e, route.id, index)}
              onClick={() => isStartOrEnd && handleExtendRoute(point)}
            >
              <ControlPointPin
                user_email={activeDrag?.user_email || null}
                isMine={isBeingDraggedByMe}
                onDelete={() => onDelete(route.id, index)}
              />
            </Marker>
          )
        })
      )}
    </>
  )
}
