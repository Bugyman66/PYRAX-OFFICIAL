'use client';

import { useRef, useState, useEffect } from 'react';
import { Stage, Layer, Rect, Circle, Arrow, Line, Group, Text } from 'react-konva';
import type Konva from 'konva';

export type AnnotationTool = 'select' | 'pin' | 'rectangle' | 'arrow' | 'freehand';

export interface AnnotationGeometry {
  type: 'pin' | 'rectangle' | 'arrow' | 'freehand';
  x: number;
  y: number;
  width?: number;
  height?: number;
  points?: number[];
  endX?: number;
  endY?: number;
}

export interface Annotation {
  id: string;
  geometryJson: AnnotationGeometry;
  color: string;
  pageOrTimecode: string | null;
  createdBy: { id: string; name: string | null; email: string };
  createdAt: string;
  comments: Array<{ id: string; body: string }>;
}

interface AnnotationOverlayProps {
  width: number;
  height: number;
  annotations: Annotation[];
  selectedTool: AnnotationTool;
  selectedColor: string;
  selectedAnnotationId: string | null;
  zoom: number;
  onAnnotationCreate: (geometry: AnnotationGeometry) => void;
  onAnnotationSelect: (annotationId: string | null) => void;
  disabled?: boolean;
}

export function AnnotationOverlay({
  width,
  height,
  annotations,
  selectedTool,
  selectedColor,
  selectedAnnotationId,
  zoom,
  onAnnotationCreate,
  onAnnotationSelect,
  disabled = false,
}: AnnotationOverlayProps) {
  const stageRef = useRef<Konva.Stage>(null);
  const [isDrawing, setIsDrawing] = useState(false);
  const [currentShape, setCurrentShape] = useState<AnnotationGeometry | null>(null);
  const [startPos, setStartPos] = useState({ x: 0, y: 0 });

  const scale = zoom / 100;

  const handleMouseDown = (e: Konva.KonvaEventObject<MouseEvent>) => {
    if (disabled || selectedTool === 'select') return;

    const stage = stageRef.current;
    if (!stage) return;

    const pos = stage.getPointerPosition();
    if (!pos) return;

    // Adjust for zoom
    const x = pos.x / scale;
    const y = pos.y / scale;

    setIsDrawing(true);
    setStartPos({ x, y });

    if (selectedTool === 'pin') {
      const geometry: AnnotationGeometry = { type: 'pin', x, y };
      onAnnotationCreate(geometry);
      setIsDrawing(false);
      return;
    }

    if (selectedTool === 'freehand') {
      setCurrentShape({ type: 'freehand', x: 0, y: 0, points: [x, y] });
    } else if (selectedTool === 'rectangle') {
      setCurrentShape({ type: 'rectangle', x, y, width: 0, height: 0 });
    } else if (selectedTool === 'arrow') {
      setCurrentShape({ type: 'arrow', x, y, endX: x, endY: y });
    }
  };

  const handleMouseMove = (e: Konva.KonvaEventObject<MouseEvent>) => {
    if (!isDrawing || !currentShape || disabled) return;

    const stage = stageRef.current;
    if (!stage) return;

    const pos = stage.getPointerPosition();
    if (!pos) return;

    const x = pos.x / scale;
    const y = pos.y / scale;

    if (currentShape.type === 'freehand' && currentShape.points) {
      setCurrentShape({
        ...currentShape,
        points: [...currentShape.points, x, y],
      });
    } else if (currentShape.type === 'rectangle') {
      setCurrentShape({
        ...currentShape,
        width: x - startPos.x,
        height: y - startPos.y,
      });
    } else if (currentShape.type === 'arrow') {
      setCurrentShape({
        ...currentShape,
        endX: x,
        endY: y,
      });
    }
  };

  const handleMouseUp = () => {
    if (!isDrawing || !currentShape || disabled) return;

    setIsDrawing(false);

    // Validate shape has meaningful size
    if (currentShape.type === 'rectangle') {
      if (Math.abs(currentShape.width || 0) > 5 && Math.abs(currentShape.height || 0) > 5) {
        onAnnotationCreate(currentShape);
      }
    } else if (currentShape.type === 'arrow') {
      const dx = (currentShape.endX || 0) - currentShape.x;
      const dy = (currentShape.endY || 0) - currentShape.y;
      if (Math.sqrt(dx * dx + dy * dy) > 10) {
        onAnnotationCreate(currentShape);
      }
    } else if (currentShape.type === 'freehand' && currentShape.points && currentShape.points.length > 4) {
      onAnnotationCreate(currentShape);
    }

    setCurrentShape(null);
  };

  const handleStageClick = (e: Konva.KonvaEventObject<MouseEvent>) => {
    if (selectedTool !== 'select') return;
    
    // Click on stage background = deselect
    if (e.target === e.target.getStage()) {
      onAnnotationSelect(null);
    }
  };

  const renderAnnotation = (annotation: Annotation, isSelected: boolean) => {
    const { geometryJson, color, id } = annotation;
    const strokeWidth = isSelected ? 3 : 2;
    const opacity = isSelected ? 1 : 0.8;

    const handleClick = () => {
      if (selectedTool === 'select') {
        onAnnotationSelect(id);
      }
    };

    if (geometryJson.type === 'pin') {
      return (
        <Group key={id} onClick={handleClick}>
          <Circle
            x={geometryJson.x}
            y={geometryJson.y}
            radius={isSelected ? 14 : 12}
            fill={color}
            stroke={isSelected ? '#fff' : color}
            strokeWidth={strokeWidth}
            opacity={opacity}
            shadowColor="black"
            shadowBlur={4}
            shadowOpacity={0.3}
          />
          <Text
            x={geometryJson.x - 4}
            y={geometryJson.y - 6}
            text={String(annotations.indexOf(annotation) + 1)}
            fontSize={12}
            fill="#fff"
            fontStyle="bold"
          />
        </Group>
      );
    }

    if (geometryJson.type === 'rectangle') {
      return (
        <Rect
          key={id}
          x={geometryJson.x}
          y={geometryJson.y}
          width={geometryJson.width}
          height={geometryJson.height}
          stroke={color}
          strokeWidth={strokeWidth}
          fill={`${color}20`}
          opacity={opacity}
          onClick={handleClick}
        />
      );
    }

    if (geometryJson.type === 'arrow') {
      return (
        <Arrow
          key={id}
          points={[geometryJson.x, geometryJson.y, geometryJson.endX || 0, geometryJson.endY || 0]}
          stroke={color}
          strokeWidth={strokeWidth}
          fill={color}
          pointerLength={10}
          pointerWidth={8}
          opacity={opacity}
          onClick={handleClick}
        />
      );
    }

    if (geometryJson.type === 'freehand' && geometryJson.points) {
      return (
        <Line
          key={id}
          points={geometryJson.points}
          stroke={color}
          strokeWidth={strokeWidth}
          tension={0.5}
          lineCap="round"
          lineJoin="round"
          opacity={opacity}
          onClick={handleClick}
        />
      );
    }

    return null;
  };

  const renderCurrentShape = () => {
    if (!currentShape) return null;

    if (currentShape.type === 'rectangle') {
      return (
        <Rect
          x={currentShape.x}
          y={currentShape.y}
          width={currentShape.width}
          height={currentShape.height}
          stroke={selectedColor}
          strokeWidth={2}
          fill={`${selectedColor}20`}
          dash={[5, 5]}
        />
      );
    }

    if (currentShape.type === 'arrow') {
      return (
        <Arrow
          points={[currentShape.x, currentShape.y, currentShape.endX || 0, currentShape.endY || 0]}
          stroke={selectedColor}
          strokeWidth={2}
          fill={selectedColor}
          pointerLength={10}
          pointerWidth={8}
          dash={[5, 5]}
        />
      );
    }

    if (currentShape.type === 'freehand' && currentShape.points) {
      return (
        <Line
          points={currentShape.points}
          stroke={selectedColor}
          strokeWidth={2}
          tension={0.5}
          lineCap="round"
          lineJoin="round"
        />
      );
    }

    return null;
  };

  return (
    <Stage
      ref={stageRef}
      width={width * scale}
      height={height * scale}
      scaleX={scale}
      scaleY={scale}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
      onClick={handleStageClick}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        pointerEvents: disabled ? 'none' : 'auto',
        cursor: selectedTool === 'select' ? 'default' : 'crosshair',
      }}
    >
      <Layer>
        {annotations.map((ann) => renderAnnotation(ann, ann.id === selectedAnnotationId))}
        {renderCurrentShape()}
      </Layer>
    </Stage>
  );
}
