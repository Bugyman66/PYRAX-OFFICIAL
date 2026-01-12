'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
import { Loader2 } from 'lucide-react';

interface PdfViewerProps {
  src: string;
  zoom: number;
  rotation: number;
  currentPage: number;
  onPageChange: (page: number) => void;
  onTotalPagesChange: (total: number) => void;
}

export function PdfViewer({
  src,
  zoom,
  rotation,
  currentPage,
  onPageChange,
  onTotalPagesChange,
}: PdfViewerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [pdfDoc, setPdfDoc] = useState<any>(null);

  const renderPage = useCallback(async (pageNum: number) => {
    if (!pdfDoc || !canvasRef.current) return;

    try {
      const page = await pdfDoc.getPage(pageNum);
      const canvas = canvasRef.current;
      const context = canvas.getContext('2d');
      if (!context) return;

      const scale = zoom / 100;
      const viewport = page.getViewport({ scale, rotation });

      canvas.height = viewport.height;
      canvas.width = viewport.width;

      await page.render({
        canvasContext: context,
        viewport,
      }).promise;

      setLoading(false);
    } catch (err) {
      console.error('Failed to render page:', err);
      setError('Failed to render page');
    }
  }, [pdfDoc, zoom, rotation]);

  useEffect(() => {
    const loadPdf = async () => {
      try {
        setLoading(true);
        setError(null);

        const pdfjsLib = await import('pdfjs-dist');
        
        // Set worker path
        pdfjsLib.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.mjs';

        const loadingTask = pdfjsLib.getDocument(src);
        const pdf = await loadingTask.promise;
        
        setPdfDoc(pdf);
        onTotalPagesChange(pdf.numPages);
      } catch (err) {
        console.error('Failed to load PDF:', err);
        setError('Failed to load PDF');
        setLoading(false);
      }
    };

    loadPdf();
  }, [src, onTotalPagesChange]);

  useEffect(() => {
    if (pdfDoc) {
      renderPage(currentPage);
    }
  }, [pdfDoc, currentPage, renderPage]);

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-red-400 mb-2">{error}</p>
          <p className="text-stone-500 text-sm">Try downloading the file instead</p>
        </div>
      </div>
    );
  }

  return (
    <div ref={containerRef} className="flex items-center justify-center h-full overflow-auto">
      {loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-stone-950/50">
          <Loader2 className="w-8 h-8 text-stone-500 animate-spin" />
        </div>
      )}
      <canvas
        ref={canvasRef}
        className="max-w-none shadow-2xl"
        style={{ display: loading ? 'none' : 'block' }}
      />
    </div>
  );
}
