import mapboxgl from 'mapbox-gl'

export const initializeMapbox = (token: string) => {
  mapboxgl.accessToken = token
}

export const getConfidenceColor = (confidence: number): string => {
  const colors = {
    5: '#22c55e', // green-500
    4: '#84cc16', // lime-500
    3: '#eab308', // yellow-500
    2: '#f97316', // orange-500
    1: '#ef4444', // red-500
  }
  return colors[confidence as keyof typeof colors] || '#6b7280'
}

export const createTrackLayerStyle = (sourceId: string) => ({
  id: `${sourceId}-layer`,
  type: 'line' as const,
  source: sourceId,
  paint: {
    'line-color': [
      'case',
      ['==', ['get', 'confidence'], 5], '#22c55e',
      ['==', ['get', 'confidence'], 4], '#84cc16',
      ['==', ['get', 'confidence'], 3], '#eab308',
      ['==', ['get', 'confidence'], 2], '#f97316',
      ['==', ['get', 'confidence'], 1], '#ef4444',
      '#6b7280'
    ],
    'line-width': [
      'interpolate',
      ['linear'],
      ['zoom'],
      5, 1,
      10, 2,
      15, 3
    ],
    'line-opacity': 0.8,
  },
})
