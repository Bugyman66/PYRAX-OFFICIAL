'use client';

import { useState, useRef, useEffect } from 'react';
import { Loader2 } from 'lucide-react';

interface ImageViewerProps {
  src: string;
  alt: string;
  zoom: number;
  rotation: number;
}

export function ImageViewer({ src, alt, zoom, rotation }: ImageViewerProps) {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [dragging, setDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Reset position when zoom changes
    if (zoom <= 100) {
      setPosition({ x: 0, y: 0 });
    }
  }, [zoom]);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (zoom > 100) {
      setDragging(true);
      setDragStart({ x: e.clientX - position.x, y: e.clientY - position.y });
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (dragging) {
      setPosition({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y,
      });
    }
  };

  const handleMouseUp = () => {
    setDragging(false);
  };

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-red-400 mb-2">Failed to load image</p>
          <p className="text-stone-500 text-sm">The image could not be displayed</p>
        </div>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className={`flex items-center justify-center h-full overflow-hidden ${
        zoom > 100 ? 'cursor-grab' : ''
      } ${dragging ? 'cursor-grabbing' : ''}`}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    >
      {loading && (
        <div className="absolute inset-0 flex items-center justify-center">
          <Loader2 className="w-8 h-8 text-stone-500 animate-spin" />
        </div>
      )}
      <img
        src={src}
        alt={alt}
        className="max-w-none transition-transform duration-100 select-none"
        style={{
          transform: `translate(${position.x}px, ${position.y}px) scale(${zoom / 100}) rotate(${rotation}deg)`,
          maxHeight: zoom <= 100 ? 'calc(100vh - 120px)' : 'none',
          opacity: loading ? 0 : 1,
        }}
        onLoad={() => setLoading(false)}
        onError={() => {
          setLoading(false);
          setError(true);
        }}
        draggable={false}
      />
    </div>
  );
}
