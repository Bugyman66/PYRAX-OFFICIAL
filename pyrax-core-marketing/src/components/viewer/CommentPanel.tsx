'use client';

import { useState } from 'react';
import { MessageSquare, Send, X, MoreVertical, Trash2, Check, Reply, ChevronDown, ChevronUp } from 'lucide-react';

interface User {
  id: string;
  name: string | null;
  email: string;
}

interface Comment {
  id: string;
  body: string;
  resolved: boolean;
  createdBy: User;
  createdAt: string;
  annotation?: { id: string; color: string } | null;
  replies?: Comment[];
}

interface CommentPanelProps {
  comments: Comment[];
  selectedAnnotationId: string | null;
  currentUserId: string;
  onAddComment: (body: string, annotationId?: string, parentId?: string) => Promise<void>;
  onResolveComment: (commentId: string, resolved: boolean) => Promise<void>;
  onDeleteComment: (commentId: string) => Promise<void>;
  onAnnotationSelect: (annotationId: string | null) => void;
}

export function CommentPanel({
  comments,
  selectedAnnotationId,
  currentUserId,
  onAddComment,
  onResolveComment,
  onDeleteComment,
  onAnnotationSelect,
}: CommentPanelProps) {
  const [newComment, setNewComment] = useState('');
  const [replyingTo, setReplyingTo] = useState<string | null>(null);
  const [replyText, setReplyText] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [expandedComments, setExpandedComments] = useState<Set<string>>(new Set());
  const [menuOpen, setMenuOpen] = useState<string | null>(null);

  const filteredComments = selectedAnnotationId
    ? comments.filter((c) => c.annotation?.id === selectedAnnotationId)
    : comments.filter((c) => !c.annotation);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newComment.trim() || submitting) return;

    setSubmitting(true);
    try {
      await onAddComment(newComment.trim(), selectedAnnotationId || undefined);
      setNewComment('');
    } finally {
      setSubmitting(false);
    }
  };

  const handleReply = async (parentId: string) => {
    if (!replyText.trim() || submitting) return;

    setSubmitting(true);
    try {
      await onAddComment(replyText.trim(), undefined, parentId);
      setReplyText('');
      setReplyingTo(null);
    } finally {
      setSubmitting(false);
    }
  };

  const toggleExpand = (commentId: string) => {
    const newExpanded = new Set(expandedComments);
    if (newExpanded.has(commentId)) {
      newExpanded.delete(commentId);
    } else {
      newExpanded.add(commentId);
    }
    setExpandedComments(newExpanded);
  };

  const formatTime = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  };

  const renderComment = (comment: Comment, isReply = false) => {
    const isOwn = comment.createdBy.id === currentUserId;
    const hasReplies = comment.replies && comment.replies.length > 0;
    const isExpanded = expandedComments.has(comment.id);

    return (
      <div
        key={comment.id}
        className={`${isReply ? 'ml-6 border-l-2 border-stone-700 pl-4' : ''} ${
          comment.resolved ? 'opacity-60' : ''
        }`}
      >
        <div className="p-3 bg-stone-800/50 rounded-lg hover:bg-stone-800 transition-colors">
          <div className="flex items-start justify-between gap-2">
            <div className="flex items-center gap-2 min-w-0">
              {comment.annotation && (
                <button
                  onClick={() => onAnnotationSelect(comment.annotation!.id)}
                  className="w-3 h-3 rounded-full flex-shrink-0"
                  style={{ backgroundColor: comment.annotation.color }}
                  title="Go to annotation"
                />
              )}
              <span className="text-sm font-medium text-stone-200 truncate">
                {comment.createdBy.name || comment.createdBy.email.split('@')[0]}
              </span>
              <span className="text-xs text-stone-500">{formatTime(comment.createdAt)}</span>
              {comment.resolved && (
                <span className="text-xs px-1.5 py-0.5 bg-green-500/20 text-green-400 rounded">
                  Resolved
                </span>
              )}
            </div>
            
            <div className="relative flex-shrink-0">
              <button
                onClick={() => setMenuOpen(menuOpen === comment.id ? null : comment.id)}
                className="p-1 text-stone-500 hover:text-stone-300 rounded"
              >
                <MoreVertical className="w-4 h-4" />
              </button>
              {menuOpen === comment.id && (
                <div className="absolute right-0 top-full mt-1 w-32 bg-stone-800 border border-stone-700 rounded-lg shadow-xl z-10">
                  <button
                    onClick={() => {
                      onResolveComment(comment.id, !comment.resolved);
                      setMenuOpen(null);
                    }}
                    className="flex items-center gap-2 px-3 py-2 text-sm text-stone-300 hover:bg-stone-700 w-full rounded-t-lg"
                  >
                    <Check className="w-4 h-4" />
                    {comment.resolved ? 'Unresolve' : 'Resolve'}
                  </button>
                  {isOwn && (
                    <button
                      onClick={() => {
                        onDeleteComment(comment.id);
                        setMenuOpen(null);
                      }}
                      className="flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-stone-700 w-full rounded-b-lg"
                    >
                      <Trash2 className="w-4 h-4" />
                      Delete
                    </button>
                  )}
                </div>
              )}
            </div>
          </div>

          <p className="text-sm text-stone-300 mt-2 whitespace-pre-wrap">{comment.body}</p>

          <div className="flex items-center gap-3 mt-2">
            {!isReply && (
              <button
                onClick={() => setReplyingTo(replyingTo === comment.id ? null : comment.id)}
                className="text-xs text-stone-500 hover:text-stone-300 flex items-center gap-1"
              >
                <Reply className="w-3 h-3" />
                Reply
              </button>
            )}
            {hasReplies && (
              <button
                onClick={() => toggleExpand(comment.id)}
                className="text-xs text-stone-500 hover:text-stone-300 flex items-center gap-1"
              >
                {isExpanded ? <ChevronUp className="w-3 h-3" /> : <ChevronDown className="w-3 h-3" />}
                {comment.replies!.length} {comment.replies!.length === 1 ? 'reply' : 'replies'}
              </button>
            )}
          </div>
        </div>

        {/* Reply input */}
        {replyingTo === comment.id && (
          <div className="mt-2 ml-6 flex gap-2">
            <input
              type="text"
              value={replyText}
              onChange={(e) => setReplyText(e.target.value)}
              placeholder="Write a reply..."
              className="flex-1 px-3 py-1.5 bg-stone-800 border border-stone-700 rounded-lg text-sm text-stone-50 placeholder:text-stone-500 focus:outline-none focus:border-pyrax-500"
              onKeyDown={(e) => e.key === 'Enter' && handleReply(comment.id)}
            />
            <button
              onClick={() => handleReply(comment.id)}
              disabled={!replyText.trim() || submitting}
              className="p-1.5 bg-pyrax-500 text-white rounded-lg hover:bg-pyrax-600 disabled:opacity-50"
            >
              <Send className="w-4 h-4" />
            </button>
            <button
              onClick={() => { setReplyingTo(null); setReplyText(''); }}
              className="p-1.5 text-stone-500 hover:text-stone-300"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        )}

        {/* Replies */}
        {hasReplies && isExpanded && (
          <div className="mt-2 space-y-2">
            {comment.replies!.map((reply) => renderComment(reply, true))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="w-80 bg-stone-900 border-l border-stone-800 flex flex-col h-full">
      <div className="p-4 border-b border-stone-800">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <MessageSquare className="w-5 h-5 text-stone-400" />
            <h2 className="font-semibold text-stone-50">Comments</h2>
          </div>
          <span className="text-xs text-stone-500 bg-stone-800 px-2 py-1 rounded-full">
            {filteredComments.length}
          </span>
        </div>
        {selectedAnnotationId && (
          <div className="mt-2 flex items-center gap-2">
            <span className="text-xs text-stone-500">Showing comments for selected annotation</span>
            <button
              onClick={() => onAnnotationSelect(null)}
              className="text-xs text-pyrax-400 hover:text-pyrax-300"
            >
              Show all
            </button>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {filteredComments.length === 0 ? (
          <div className="text-center py-8">
            <MessageSquare className="w-10 h-10 text-stone-700 mx-auto mb-3" />
            <p className="text-stone-500 text-sm">No comments yet</p>
            <p className="text-stone-600 text-xs mt-1">
              {selectedAnnotationId
                ? 'Add a comment to this annotation'
                : 'Start a discussion or add annotations'}
            </p>
          </div>
        ) : (
          filteredComments.map((comment) => renderComment(comment))
        )}
      </div>

      <form onSubmit={handleSubmit} className="p-4 border-t border-stone-800">
        <div className="flex gap-2">
          <input
            type="text"
            value={newComment}
            onChange={(e) => setNewComment(e.target.value)}
            placeholder={selectedAnnotationId ? 'Comment on annotation...' : 'Add a comment...'}
            className="flex-1 px-3 py-2 bg-stone-800 border border-stone-700 rounded-lg text-sm text-stone-50 placeholder:text-stone-500 focus:outline-none focus:border-pyrax-500"
          />
          <button
            type="submit"
            disabled={!newComment.trim() || submitting}
            className="p-2 pyrax-gradient text-white rounded-lg hover:opacity-90 disabled:opacity-50"
          >
            <Send className="w-5 h-5" />
          </button>
        </div>
      </form>
    </div>
  );
}
