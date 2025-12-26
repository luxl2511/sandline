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
  userId: string
  userEmail: string
  routeId: string
  pointIndex: number
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
        const key = `${msg.data.routeId}-${msg.data.pointIndex}`
        setActiveDrags(prev => {
          const next = new Map(prev)
          next.set(key, {
            userId: msg.userId,
            userEmail: msg.userEmail,
            routeId: msg.data.routeId,
            pointIndex: msg.data.pointIndex,
          })
          return next
        })
      } else if (msg.type === 'drag_end') {
        const key = `${msg.data.routeId}-${msg.data.pointIndex}`
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
      next.set(key, { userId: user.id, userEmail: user.email || '', routeId, pointIndex })
      return next
    })
    broadcast('drag_start', { routeId, pointIndex })
  }

  const onDrag = (e: any, routeId: string, pointIndex: number) => {
    if (!user) return
    broadcast('drag_update', {
      routeId,
      pointIndex,
      newPosition: [e.lngLat.lng, e.lngLat.lat],
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
      routeId,
      pointIndex,
      newPosition: [e.lngLat.lng, e.lngLat.lat],
    })

    const route = routes.find(r => r.id === routeId)
    if (!route || !route.controlPoints) return

    const newControlPoints = [...route.controlPoints]
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
    if (!route || !route.controlPoints) return

    const newControlPoints = [...route.controlPoints]
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
        route.controlPoints?.map((point, index) => {
          const key = `${route.id}-${index}`
          const activeDrag = activeDrags.get(key)
          const isBeingDraggedByMe = activeDrag?.userId === user?.id
          const isBeingDraggedByOther = activeDrag && activeDrag.userId !== user?.id
          const isStartOrEnd = index === 0 || index === (route.controlPoints?.length || 0) - 1

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
                userEmail={activeDrag?.userEmail || null}
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
