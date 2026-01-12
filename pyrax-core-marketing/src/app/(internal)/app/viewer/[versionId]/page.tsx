'use client';

import { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  ArrowLeft,
  Download,
  ChevronDown,
  ZoomIn,
  ZoomOut,
  RotateCw,
  Maximize2,
  MessageSquare,
  Loader2,
  FileText,
  Image as ImageIcon,
  Video,
  File,
  ChevronLeft,
  ChevronRight,
  PenTool,
  X,
  GitBranch,
  Globe,
} from 'lucide-react';
import { getViewerType, formatMimeType } from '@/lib/file-types';
import { ImageViewer } from '@/components/viewer/ImageViewer';
import { VideoViewer } from '@/components/viewer/VideoViewer';
import { PdfViewer } from '@/components/viewer/PdfViewer';
import { AnnotationToolbar } from '@/components/viewer/AnnotationToolbar';
import { CommentPanel } from '@/components/viewer/CommentPanel';
import WorkflowPanel from '@/components/viewer/WorkflowPanel';
import PublishModal from '@/components/viewer/PublishModal';
import { AnnotationOverlay } from '@/components/viewer/AnnotationOverlay';
import type { AnnotationTool, AnnotationGeometry, Annotation } from '@/components/viewer/AnnotationOverlay';

interface Comment {
  id: string;
  body: string;
  resolved: boolean;
  createdBy: { id: string; name: string | null; email: string };
  createdAt: string;
  annotation?: { id: string; color: string } | null;
  replies?: Comment[];
}

interface Version {
  id: string;
  versionNumber: number;
  driveFileId: string;
  fileName: string;
  mimeType: string;
  fileSize: number | null;
  kind: string;
  createdBy: { id: string; name: string | null; email: string };
  createdAt: string;
  proof: {
    id: string;
    title: string;
    folder: { id: string; name: string };
    versions: Array<{ id: string; versionNumber: number; kind: string }>;
  };
}

export default function ViewerPage({ params }: { params: { versionId: string } }) {
  const { versionId } = params;
  const router = useRouter();
  const [version, setVersion] = useState<Version | null>(null);
  const [loading, setLoading] = useState(true);
  const [zoom, setZoom] = useState(100);
  const [rotation, setRotation] = useState(0);
  const [showVersionDropdown, setShowVersionDropdown] = useState(false);
  const [pdfPage, setPdfPage] = useState(1);
  const [pdfTotalPages, setPdfTotalPages] = useState(1);
  const [videoTime, setVideoTime] = useState(0);
  const [videoDuration, setVideoDuration] = useState(0);
  
  // Annotation & Comment state
  const [showComments, setShowComments] = useState(false);
  const [showWorkflow, setShowWorkflow] = useState(false);
  const [showAnnotationTools, setShowAnnotationTools] = useState(false);
  const [showPublishModal, setShowPublishModal] = useState(false);
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [comments, setComments] = useState<Comment[]>([]);
  const [selectedTool, setSelectedTool] = useState<AnnotationTool>('select');
  const [selectedColor, setSelectedColor] = useState('#FF5500');
  const [selectedAnnotationId, setSelectedAnnotationId] = useState<string | null>(null);
  const [currentUserId, setCurrentUserId] = useState<string>('');

  const fetchVersion = useCallback(async () => {
    try {
      const res = await fetch(`/api/versions/${versionId}`);
      if (res.ok) {
        const data = await res.json();
        setVersion(data.version);
      } else {
        router.push('/app/folders');
      }
    } catch (error) {
      console.error('Failed to fetch version:', error);
    } finally {
      setLoading(false);
    }
  }, [versionId, router]);

  const fetchAnnotations = useCallback(async () => {
    try {
      const res = await fetch(`/api/versions/${versionId}/annotations`);
      if (res.ok) {
        const data = await res.json();
        setAnnotations(data.annotations || []);
      }
    } catch (error) {
      console.error('Failed to fetch annotations:', error);
    }
  }, [versionId]);

  const fetchComments = useCallback(async () => {
    try {
      const res = await fetch(`/api/versions/${versionId}/comments`);
      if (res.ok) {
        const data = await res.json();
        setComments(data.comments || []);
      }
    } catch (error) {
      console.error('Failed to fetch comments:', error);
    }
  }, [versionId]);

  useEffect(() => {
    fetchVersion();
    fetchAnnotations();
    fetchComments();
  }, [fetchVersion, fetchAnnotations, fetchComments]);

  const handleCreateAnnotation = async (geometry: AnnotationGeometry) => {
    try {
      const pageOrTimecode = pdfPage > 1 ? `page:${pdfPage}` : videoTime > 0 ? `time:${videoTime}` : null;
      
      const res = await fetch(`/api/versions/${versionId}/annotations`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          geometryJson: geometry,
          color: selectedColor,
          pageOrTimecode,
        }),
      });

      if (res.ok) {
        fetchAnnotations();
        setSelectedTool('select');
      }
    } catch (error) {
      console.error('Failed to create annotation:', error);
    }
  };

  const handleAddComment = async (body: string, annotationId?: string, parentId?: string) => {
    const res = await fetch(`/api/versions/${versionId}/comments`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ body, annotationId, parentId }),
    });

    if (res.ok) {
      fetchComments();
      if (annotationId) fetchAnnotations();
    }
  };

  const handleResolveComment = async (commentId: string, resolved: boolean) => {
    await fetch(`/api/comments/${commentId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ resolved }),
    });
    fetchComments();
  };

  const handleDeleteComment = async (commentId: string) => {
    await fetch(`/api/comments/${commentId}`, { method: 'DELETE' });
    fetchComments();
  };

  const handleZoomIn = () => setZoom((z) => Math.min(z + 25, 300));
  const handleZoomOut = () => setZoom((z) => Math.max(z - 25, 25));
  const handleRotate = () => setRotation((r) => (r + 90) % 360);
  const handleFitToScreen = () => setZoom(100);

  const handleVersionChange = (newVersionId: string) => {
    setShowVersionDropdown(false);
    router.push(`/app/viewer/${newVersionId}`);
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  if (loading) {
    return (
      <div className="fixed inset-0 z-[100] bg-stone-950 flex items-center justify-center">
        <Loader2 className="w-8 h-8 text-stone-500 animate-spin" />
      </div>
    );
  }

  if (!version) {
    return (
      <div className="fixed inset-0 z-[100] bg-stone-950 flex items-center justify-center">
        <p className="text-stone-400">Version not found</p>
      </div>
    );
  }

  const viewerType = getViewerType(version.mimeType);
  const streamUrl = `/api/drive/stream/${version.driveFileId}`;

  return (
    <div className="fixed inset-0 z-[100] bg-stone-950 flex flex-col">
      {/* Toolbar */}
      <div className="h-14 bg-stone-900 border-b border-stone-800 flex items-center justify-between px-4 flex-shrink-0">
        <div className="flex items-center gap-4">
          <Link
            href={`/app/proofs/${version.proof.id}`}
            className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors"
          >
            <ArrowLeft className="w-5 h-5" />
          </Link>
          
          <div className="flex items-center gap-2">
            {viewerType === 'pdf' && <FileText className="w-5 h-5 text-red-400" />}
            {viewerType === 'image' && <ImageIcon className="w-5 h-5 text-blue-400" />}
            {viewerType === 'video' && <Video className="w-5 h-5 text-purple-400" />}
            {viewerType === 'unsupported' && <File className="w-5 h-5 text-stone-400" />}
            
            <div>
              <h1 className="text-stone-50 font-medium text-sm">{version.proof.title}</h1>
              <p className="text-stone-500 text-xs">{version.fileName}</p>
            </div>
          </div>

          {/* Version Selector */}
          <div className="relative">
            <button
              onClick={() => setShowVersionDropdown(!showVersionDropdown)}
              className="flex items-center gap-2 px-3 py-1.5 bg-stone-800 hover:bg-stone-700 rounded-lg text-sm text-stone-300 transition-colors"
            >
              v{version.versionNumber}
              {version.kind === 'Rendition' && (
                <span className="text-xs text-blue-400">(Rendition)</span>
              )}
              <ChevronDown className="w-4 h-4" />
            </button>
            
            {showVersionDropdown && (
              <div className="absolute top-full left-0 mt-1 w-48 bg-stone-800 border border-stone-700 rounded-lg shadow-xl z-20">
                {version.proof.versions.map((v) => (
                  <button
                    key={v.id}
                    onClick={() => handleVersionChange(v.id)}
                    className={`w-full px-3 py-2 text-left text-sm hover:bg-stone-700 transition-colors first:rounded-t-lg last:rounded-b-lg ${
                      v.id === version.id ? 'text-pyrax-400' : 'text-stone-300'
                    }`}
                  >
                    Version {v.versionNumber}
                    {v.kind === 'Rendition' && (
                      <span className="text-xs text-blue-400 ml-2">(Rendition)</span>
                    )}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Center Controls */}
        <div className="flex items-center gap-2">
          {viewerType === 'pdf' && (
            <>
              <button
                onClick={() => setPdfPage((p) => Math.max(1, p - 1))}
                disabled={pdfPage <= 1}
                className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors disabled:opacity-50"
              >
                <ChevronLeft className="w-4 h-4" />
              </button>
              <span className="text-stone-300 text-sm min-w-[80px] text-center">
                {pdfPage} / {pdfTotalPages}
              </span>
              <button
                onClick={() => setPdfPage((p) => Math.min(pdfTotalPages, p + 1))}
                disabled={pdfPage >= pdfTotalPages}
                className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors disabled:opacity-50"
              >
                <ChevronRight className="w-4 h-4" />
              </button>
              <div className="w-px h-6 bg-stone-700 mx-2" />
            </>
          )}

          {viewerType === 'video' && (
            <>
              <span className="text-stone-300 text-sm font-mono">
                {formatTime(videoTime)} / {formatTime(videoDuration)}
              </span>
              <div className="w-px h-6 bg-stone-700 mx-2" />
            </>
          )}

          {(viewerType === 'image' || viewerType === 'pdf') && (
            <>
              <button
                onClick={handleZoomOut}
                className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors"
                title="Zoom Out"
              >
                <ZoomOut className="w-4 h-4" />
              </button>
              <span className="text-stone-300 text-sm min-w-[50px] text-center">{zoom}%</span>
              <button
                onClick={handleZoomIn}
                className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors"
                title="Zoom In"
              >
                <ZoomIn className="w-4 h-4" />
              </button>
              <button
                onClick={handleFitToScreen}
                className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors"
                title="Fit to Screen"
              >
                <Maximize2 className="w-4 h-4" />
              </button>
              <button
                onClick={handleRotate}
                className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors"
                title="Rotate"
              >
                <RotateCw className="w-4 h-4" />
              </button>
            </>
          )}
        </div>

        {/* Right Controls */}
        <div className="flex items-center gap-2">
          <span className="text-xs text-stone-500 px-2 py-1 bg-stone-800 rounded">
            {formatMimeType(version.mimeType)}
          </span>
          
          <button
            onClick={() => setShowAnnotationTools(!showAnnotationTools)}
            className={`p-2 rounded-lg transition-colors ${
              showAnnotationTools
                ? 'bg-pyrax-500/20 text-pyrax-400'
                : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
            }`}
            title="Annotation Tools"
          >
            <PenTool className="w-5 h-5" />
          </button>
          
          <button
            onClick={() => setShowComments(!showComments)}
            className={`p-2 rounded-lg transition-colors ${
              showComments
                ? 'bg-pyrax-500/20 text-pyrax-400'
                : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
            }`}
            title="Comments"
          >
            <MessageSquare className="w-5 h-5" />
            {comments.length > 0 && (
              <span className="absolute -top-1 -right-1 w-4 h-4 bg-pyrax-500 text-white text-xs rounded-full flex items-center justify-center">
                {comments.length}
              </span>
            )}
          </button>
          
          <button
            onClick={() => setShowWorkflow(!showWorkflow)}
            className={`p-2 rounded-lg transition-colors ${
              showWorkflow
                ? 'bg-pyrax-500/20 text-pyrax-400'
                : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
            }`}
            title="Workflow"
          >
            <GitBranch className="w-5 h-5" />
          </button>

          <button
            onClick={() => setShowPublishModal(true)}
            className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors"
            title="Publish to Public Library"
          >
            <Globe className="w-5 h-5" />
          </button>
          
          <a
            href={streamUrl}
            download={version.fileName}
            className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors"
            title="Download"
          >
            <Download className="w-5 h-5" />
          </a>
        </div>
      </div>

      {/* Annotation Toolbar */}
      {showAnnotationTools && (
        <div className="absolute top-16 left-1/2 -translate-x-1/2 z-10">
          <AnnotationToolbar
            selectedTool={selectedTool}
            selectedColor={selectedColor}
            onToolChange={setSelectedTool}
            onColorChange={setSelectedColor}
          />
        </div>
      )}

      {/* Main Content Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Viewer Content */}
        <div className="flex-1 overflow-hidden flex items-center justify-center relative">
          {viewerType === 'image' && (
            <div className="relative">
              <ImageViewer
                src={streamUrl}
                alt={version.fileName}
                zoom={zoom}
                rotation={rotation}
              />
              {showAnnotationTools && (
                <AnnotationOverlay
                  width={800}
                  height={600}
                  annotations={annotations}
                  selectedTool={selectedTool}
                  selectedColor={selectedColor}
                  selectedAnnotationId={selectedAnnotationId}
                  zoom={zoom}
                  onAnnotationCreate={handleCreateAnnotation}
                  onAnnotationSelect={setSelectedAnnotationId}
                />
              )}
            </div>
          )}

          {viewerType === 'video' && (
            <div className="relative">
              <VideoViewer
                src={streamUrl}
                onTimeUpdate={setVideoTime}
                onDurationChange={setVideoDuration}
              />
              {showAnnotationTools && (
                <AnnotationOverlay
                  width={800}
                  height={450}
                  annotations={annotations}
                  selectedTool={selectedTool}
                  selectedColor={selectedColor}
                  selectedAnnotationId={selectedAnnotationId}
                  zoom={100}
                  onAnnotationCreate={handleCreateAnnotation}
                  onAnnotationSelect={setSelectedAnnotationId}
                />
              )}
            </div>
          )}

          {viewerType === 'pdf' && (
            <div className="relative">
              <PdfViewer
                src={streamUrl}
                zoom={zoom}
                rotation={rotation}
                currentPage={pdfPage}
                onPageChange={setPdfPage}
                onTotalPagesChange={setPdfTotalPages}
              />
              {showAnnotationTools && (
                <AnnotationOverlay
                  width={800}
                  height={1000}
                  annotations={annotations.filter(a => !a.pageOrTimecode || a.pageOrTimecode === `page:${pdfPage}`)}
                  selectedTool={selectedTool}
                  selectedColor={selectedColor}
                  selectedAnnotationId={selectedAnnotationId}
                  zoom={zoom}
                  onAnnotationCreate={handleCreateAnnotation}
                  onAnnotationSelect={setSelectedAnnotationId}
                />
              )}
            </div>
          )}

          {viewerType === 'unsupported' && (
            <div className="text-center p-8">
              <File className="w-24 h-24 text-stone-600 mx-auto mb-6" />
              <h2 className="text-xl font-semibold text-stone-300 mb-2">
                Preview not available
              </h2>
              <p className="text-stone-500 mb-6 max-w-md mx-auto">
                This file type ({formatMimeType(version.mimeType)}) cannot be previewed in the browser.
                Download the file to view it in the appropriate application.
              </p>
              <a
                href={streamUrl}
                download={version.fileName}
                className="inline-flex items-center gap-2 px-6 py-3 pyrax-gradient text-white font-medium rounded-lg hover:opacity-90 transition-opacity"
              >
                <Download className="w-5 h-5" />
                Download {version.fileName}
              </a>
            </div>
          )}
        </div>

        {/* Comment Panel */}
        {showComments && (
          <CommentPanel
            comments={comments}
            selectedAnnotationId={selectedAnnotationId}
            currentUserId={currentUserId}
            onAddComment={handleAddComment}
            onResolveComment={handleResolveComment}
            onDeleteComment={handleDeleteComment}
            onAnnotationSelect={setSelectedAnnotationId}
          />
        )}

        {/* Workflow Panel */}
        {showWorkflow && version && (
          <div className="w-80 bg-stone-900 border-l border-stone-800 flex flex-col flex-shrink-0">
            <div className="p-4 border-b border-stone-800 flex items-center justify-between">
              <h3 className="font-medium text-stone-100">Workflow</h3>
              <button
                onClick={() => setShowWorkflow(false)}
                className="p-1 hover:bg-stone-800 rounded"
              >
                <X className="w-4 h-4 text-stone-400" />
              </button>
            </div>
            <WorkflowPanel
              proofId={version.proof.id}
              onWorkflowChange={fetchVersion}
            />
          </div>
        )}
      </div>

      {/* Footer Info */}
      <div className="h-10 bg-stone-900 border-t border-stone-800 flex items-center justify-between px-4 text-xs text-stone-500 flex-shrink-0">
        <span>
          Uploaded by {version.createdBy.name || version.createdBy.email} on{' '}
          {new Date(version.createdAt).toLocaleString()}
        </span>
        <span>
          {version.fileSize ? `${(version.fileSize / 1024 / 1024).toFixed(2)} MB` : 'Size unknown'}
        </span>
      </div>

      {/* Publish Modal */}
      {showPublishModal && version && (
        <PublishModal
          proofId={version.proof.id}
          proofTitle={version.proof.title}
          onClose={() => setShowPublishModal(false)}
          onPublished={() => {
            setShowPublishModal(false);
            fetchVersion();
          }}
        />
      )}
    </div>
  );
}
