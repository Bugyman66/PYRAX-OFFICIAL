declare module 'react-simple-maps' {
  import { ComponentType, ReactNode } from 'react'

  export interface ComposableMapProps {
    projection?: string
    projectionConfig?: {
      center?: [number, number]
      scale?: number
      rotate?: [number, number, number]
    }
    width?: number
    height?: number
    style?: React.CSSProperties
    className?: string
    children?: ReactNode
  }

  export interface GeographiesProps {
    geography: string | object
    children: (data: { geographies: GeographyType[] }) => ReactNode
  }

  export interface GeographyType {
    rsmKey: string
    properties: Record<string, unknown>
    geometry: object
  }

  export interface GeographyProps {
    geography: GeographyType
    style?: {
      default?: React.CSSProperties
      hover?: React.CSSProperties
      pressed?: React.CSSProperties
    }
    onMouseEnter?: (event: React.MouseEvent) => void
    onMouseLeave?: (event: React.MouseEvent) => void
    onClick?: (event: React.MouseEvent) => void
    fill?: string
    stroke?: string
    strokeWidth?: number
    className?: string
  }

  export interface MarkerProps {
    coordinates: [number, number]
    children?: ReactNode
    style?: React.CSSProperties
    className?: string
    onClick?: (event: React.MouseEvent) => void
    onMouseEnter?: (event: React.MouseEvent) => void
    onMouseLeave?: (event: React.MouseEvent) => void
  }

  export interface ZoomableGroupProps {
    center?: [number, number]
    zoom?: number
    minZoom?: number
    maxZoom?: number
    translateExtent?: [[number, number], [number, number]]
    onMoveStart?: (event: { coordinates: [number, number]; zoom: number }) => void
    onMove?: (event: { coordinates: [number, number]; zoom: number; dragging: boolean }) => void
    onMoveEnd?: (event: { coordinates: [number, number]; zoom: number }) => void
    children?: ReactNode
  }

  export interface LineProps {
    from: [number, number]
    to: [number, number]
    stroke?: string
    strokeWidth?: number
    strokeLinecap?: 'butt' | 'round' | 'square'
    className?: string
  }

  export interface AnnotationProps {
    subject: [number, number]
    dx?: number
    dy?: number
    curve?: number
    connectorProps?: object
    children?: ReactNode
  }

  export const ComposableMap: ComponentType<ComposableMapProps>
  export const Geographies: ComponentType<GeographiesProps>
  export const Geography: ComponentType<GeographyProps>
  export const Marker: ComponentType<MarkerProps>
  export const ZoomableGroup: ComponentType<ZoomableGroupProps>
  export const Line: ComponentType<LineProps>
  export const Annotation: ComponentType<AnnotationProps>
}
