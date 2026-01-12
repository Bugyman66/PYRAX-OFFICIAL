'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import {
  ClipboardCheck,
  Clock,
  CheckCircle2,
  XCircle,
  AlertCircle,
  Loader2,
  Eye,
  FolderOpen,
  User,
  Calendar,
  ArrowRight,
  FileText,
  Building2,
} from 'lucide-react';

interface Department {
  id: string;
  name: string;
}

interface Folder {
  id: string;
  name: string;
  department: Department;
}

interface Creator {
  id: string;
  name: string | null;
  email: string;
}

interface Version {
  id: string;
  versionNumber: number;
  thumbnailUrl: string | null;
}

interface PendingReview {
  id: string;
  proofId: string;
  proofTitle: string;
  proofDescription: string | null;
  proofStatus: string;
  folder: Folder;
  createdBy: Creator;
  latestVersion: Version | null;
  currentStep: {
    index: number;
    name: string;
    totalSteps: number;
  };
  workflowName: string;
  createdAt: string;
  updatedAt: string;
}

interface CompletedReview {
  id: string;
  proofId: string;
  proofTitle: string;
  proofDescription: string | null;
  proofStatus: string;
  folder: Folder;
  createdBy: Creator;
  latestVersion: Version | null;
  decision: {
    type: 'Approve' | 'NeedsChanges' | 'Reject';
    note: string | null;
    stepName: string;
    createdAt: string;
  };
  workflowName: string;
  workflowStatus: string;
}

type TabType = 'pending' | 'completed';

export default function ReviewsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('pending');
  const [pendingReviews, setPendingReviews] = useState<PendingReview[]>([]);
  const [completedReviews, setCompletedReviews] = useState<CompletedReview[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchReviews();
  }, [activeTab]);

  const fetchReviews = async () => {
    setLoading(true);
    try {
      const res = await fetch(`/api/reviews?type=${activeTab}`);
      const data = await res.json();
      
      if (activeTab === 'pending') {
        setPendingReviews(data.reviews || []);
      } else {
        setCompletedReviews(data.reviews || []);
      }
    } catch (error) {
      console.error('Failed to fetch reviews:', error);
    } finally {
      setLoading(false);
    }
  };

  const getDecisionIcon = (type: string) => {
    switch (type) {
      case 'Approve':
        return <CheckCircle2 className="w-4 h-4 text-green-500" />;
      case 'NeedsChanges':
        return <AlertCircle className="w-4 h-4 text-yellow-500" />;
      case 'Reject':
        return <XCircle className="w-4 h-4 text-red-500" />;
      default:
        return null;
    }
  };

  const getDecisionLabel = (type: string) => {
    switch (type) {
      case 'Approve':
        return 'Approved';
      case 'NeedsChanges':
        return 'Needs Changes';
      case 'Reject':
        return 'Rejected';
      default:
        return type;
    }
  };

  const getDecisionBadgeClass = (type: string) => {
    switch (type) {
      case 'Approve':
        return 'bg-green-500/10 text-green-400 border-green-500/20';
      case 'NeedsChanges':
        return 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20';
      case 'Reject':
        return 'bg-red-500/10 text-red-400 border-red-500/20';
      default:
        return 'bg-stone-500/10 text-stone-400 border-stone-500/20';
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  const formatRelativeTime = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return formatDate(dateString);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-stone-50">Reviews</h1>
          <p className="text-stone-400 mt-1">Manage your proof reviews and approvals</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-stone-900 p-1 rounded-lg w-fit">
        <button
          onClick={() => setActiveTab('pending')}
          className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            activeTab === 'pending'
              ? 'bg-pyrax-500 text-white'
              : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
          }`}
        >
          <Clock className="w-4 h-4" />
          Pending
          {pendingReviews.length > 0 && activeTab !== 'pending' && (
            <span className="ml-1 px-1.5 py-0.5 text-xs bg-pyrax-500/20 text-pyrax-400 rounded-full">
              {pendingReviews.length}
            </span>
          )}
        </button>
        <button
          onClick={() => setActiveTab('completed')}
          className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            activeTab === 'completed'
              ? 'bg-pyrax-500 text-white'
              : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
          }`}
        >
          <ClipboardCheck className="w-4 h-4" />
          Completed
        </button>
      </div>

      {/* Content */}
      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 text-stone-500 animate-spin" />
        </div>
      ) : activeTab === 'pending' ? (
        pendingReviews.length === 0 ? (
          <div className="text-center py-12 bg-stone-900 rounded-xl border border-stone-800">
            <CheckCircle2 className="w-12 h-12 text-green-500/50 mx-auto mb-4" />
            <h3 className="text-stone-300 font-medium mb-2">All caught up!</h3>
            <p className="text-stone-500 text-sm">
              You have no pending reviews at the moment.
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {pendingReviews.map((review) => (
              <div
                key={review.id}
                className="bg-stone-900 rounded-xl border border-stone-800 p-5 hover:border-stone-700 transition-colors"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-2">
                      <div className="w-10 h-10 bg-pyrax-500/10 rounded-lg flex items-center justify-center flex-shrink-0">
                        <FileText className="w-5 h-5 text-pyrax-500" />
                      </div>
                      <div className="min-w-0">
                        <h3 className="text-stone-50 font-medium truncate">
                          {review.proofTitle}
                        </h3>
                        <div className="flex items-center gap-2 text-xs text-stone-500">
                          <Building2 className="w-3 h-3" />
                          <span>{review.folder.department.name}</span>
                          <span className="text-stone-700">•</span>
                          <FolderOpen className="w-3 h-3" />
                          <span>{review.folder.name}</span>
                        </div>
                      </div>
                    </div>

                    {review.proofDescription && (
                      <p className="text-sm text-stone-400 mb-3 line-clamp-2 ml-13">
                        {review.proofDescription}
                      </p>
                    )}

                    <div className="flex flex-wrap items-center gap-3 text-sm ml-13">
                      <div className="flex items-center gap-1.5 text-stone-500">
                        <User className="w-3.5 h-3.5" />
                        <span>{review.createdBy.name || review.createdBy.email}</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-stone-500">
                        <Calendar className="w-3.5 h-3.5" />
                        <span>{formatRelativeTime(review.updatedAt)}</span>
                      </div>
                      <div className="flex items-center gap-1.5 px-2 py-0.5 bg-pyrax-500/10 text-pyrax-400 rounded-full text-xs">
                        <span>Step {review.currentStep.index + 1}/{review.currentStep.totalSteps}</span>
                        <span className="text-pyrax-500/50">•</span>
                        <span>{review.currentStep.name}</span>
                      </div>
                    </div>
                  </div>

                  <Link
                    href={`/app/viewer/${review.latestVersion?.id || review.proofId}`}
                    className="flex items-center gap-2 px-4 py-2 pyrax-gradient text-white font-medium rounded-lg hover:opacity-90 transition-opacity flex-shrink-0"
                  >
                    <Eye className="w-4 h-4" />
                    Review
                    <ArrowRight className="w-4 h-4" />
                  </Link>
                </div>
              </div>
            ))}
          </div>
        )
      ) : completedReviews.length === 0 ? (
        <div className="text-center py-12 bg-stone-900 rounded-xl border border-stone-800">
          <ClipboardCheck className="w-12 h-12 text-stone-600 mx-auto mb-4" />
          <h3 className="text-stone-300 font-medium mb-2">No completed reviews</h3>
          <p className="text-stone-500 text-sm">
            Your review history will appear here once you make decisions on proofs.
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {completedReviews.map((review) => (
            <div
              key={review.id}
              className="bg-stone-900 rounded-xl border border-stone-800 p-5 hover:border-stone-700 transition-colors"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-3 mb-2">
                    <div className="w-10 h-10 bg-stone-800 rounded-lg flex items-center justify-center flex-shrink-0">
                      {getDecisionIcon(review.decision.type)}
                    </div>
                    <div className="min-w-0">
                      <h3 className="text-stone-50 font-medium truncate">
                        {review.proofTitle}
                      </h3>
                      <div className="flex items-center gap-2 text-xs text-stone-500">
                        <Building2 className="w-3 h-3" />
                        <span>{review.folder.department.name}</span>
                        <span className="text-stone-700">•</span>
                        <FolderOpen className="w-3 h-3" />
                        <span>{review.folder.name}</span>
                      </div>
                    </div>
                  </div>

                  <div className="flex flex-wrap items-center gap-3 text-sm ml-13">
                    <div className={`flex items-center gap-1.5 px-2 py-0.5 rounded-full border text-xs ${getDecisionBadgeClass(review.decision.type)}`}>
                      {getDecisionIcon(review.decision.type)}
                      <span>{getDecisionLabel(review.decision.type)}</span>
                    </div>
                    <div className="text-stone-500 text-xs">
                      on &quot;{review.decision.stepName}&quot;
                    </div>
                    <div className="flex items-center gap-1.5 text-stone-500">
                      <Calendar className="w-3.5 h-3.5" />
                      <span>{formatDate(review.decision.createdAt)}</span>
                    </div>
                    {review.workflowStatus === 'Completed' && (
                      <span className="px-2 py-0.5 bg-green-500/10 text-green-400 rounded-full text-xs">
                        Workflow Complete
                      </span>
                    )}
                    {review.workflowStatus === 'Cancelled' && (
                      <span className="px-2 py-0.5 bg-red-500/10 text-red-400 rounded-full text-xs">
                        Workflow Cancelled
                      </span>
                    )}
                  </div>

                  {review.decision.note && (
                    <p className="text-sm text-stone-400 mt-2 ml-13 italic">
                      &quot;{review.decision.note}&quot;
                    </p>
                  )}
                </div>

                <Link
                  href={`/app/viewer/${review.latestVersion?.id || review.proofId}`}
                  className="flex items-center gap-2 px-3 py-2 bg-stone-800 hover:bg-stone-700 text-stone-300 rounded-lg transition-colors flex-shrink-0"
                >
                  <Eye className="w-4 h-4" />
                  View
                </Link>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
