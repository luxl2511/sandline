'use client'

import MapView from '@/components/map/MapView'
import LayerControls from '@/components/map/LayerControls'
import RouteEditor from '@/components/route/RouteEditor'

export default function Home() {
  return (
    <main className="relative w-full h-screen">
      <MapView />
      <div className="absolute top-4 left-4 z-10">
        <LayerControls />
      </div>
      <div className="absolute top-4 right-4 z-10">
        <RouteEditor />
      </div>
    </main>
  )
}
