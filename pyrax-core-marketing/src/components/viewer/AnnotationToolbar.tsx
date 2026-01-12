'use client';

import { MousePointer2, MapPin, Square, ArrowUpRight, Pencil } from 'lucide-react';
import type { AnnotationTool } from './AnnotationOverlay';

interface AnnotationToolbarProps {
  selectedTool: AnnotationTool;
  selectedColor: string;
  onToolChange: (tool: AnnotationTool) => void;
  onColorChange: (color: string) => void;
  disabled?: boolean;
}

const ANNOTATION_COLORS = [
  '#FF5500', // PYRAX orange
  '#EF4444', // Red
  '#F59E0B', // Amber
  '#10B981', // Green
  '#3B82F6', // Blue
  '#8B5CF6', // Purple
  '#EC4899', // Pink
];

const TOOLS: { id: AnnotationTool; icon: typeof MousePointer2; label: string }[] = [
  { id: 'select', icon: MousePointer2, label: 'Select' },
  { id: 'pin', icon: MapPin, label: 'Pin' },
  { id: 'rectangle', icon: Square, label: 'Rectangle' },
  { id: 'arrow', icon: ArrowUpRight, label: 'Arrow' },
  { id: 'freehand', icon: Pencil, label: 'Freehand' },
];

export function AnnotationToolbar({
  selectedTool,
  selectedColor,
  onToolChange,
  onColorChange,
  disabled = false,
}: AnnotationToolbarProps) {
  return (
    <div className="flex items-center gap-2 p-2 bg-stone-900 rounded-lg border border-stone-700">
      {/* Tools */}
      <div className="flex items-center gap-1">
        {TOOLS.map((tool) => (
          <button
            key={tool.id}
            onClick={() => onToolChange(tool.id)}
            disabled={disabled}
            className={`p-2 rounded-lg transition-colors ${
              selectedTool === tool.id
                ? 'bg-pyrax-500/20 text-pyrax-400'
                : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
            } disabled:opacity-50 disabled:cursor-not-allowed`}
            title={tool.label}
          >
            <tool.icon className="w-5 h-5" />
          </button>
        ))}
      </div>

      {/* Divider */}
      <div className="w-px h-6 bg-stone-700" />

      {/* Colors */}
      <div className="flex items-center gap-1">
        {ANNOTATION_COLORS.map((color) => (
          <button
            key={color}
            onClick={() => onColorChange(color)}
            disabled={disabled}
            className={`w-6 h-6 rounded-full border-2 transition-transform hover:scale-110 ${
              selectedColor === color ? 'border-white' : 'border-transparent'
            } disabled:opacity-50 disabled:cursor-not-allowed`}
            style={{ backgroundColor: color }}
            title={color}
          />
        ))}
      </div>
    </div>
  );
}
