'use client';

import { useState, useEffect } from 'react';
import { 
  Clock, 
  AlertTriangle, 
  CheckCircle, 
  FileText,
  ArrowRight,
  TrendingUp,
  Loader2,
  User,
  MessageSquare,
  GitBranch,
  FolderOpen,
  RefreshCw,
} from 'lucide-react';
import Link from 'next/link';

interface DashboardStats {
  totalProofs: number;
  inReview: number;
  approved: number;
  waitingOnMe: number;
  overdue: number;
  needsChanges: number;
}

interface ProofListItem {
  id: string;
  title: string;
  status: string;
  folder: { id: string; name: string };
  createdBy: { id: string; name: string | null; email: string };
  latestVersion?: {
    id: string;
    versionNumber: number;
    fileName: string;
    mimeType: string;
  };
  workflow?: {
    id: string;
    status: string;
    currentStepIndex: number;
    currentStepName: string | null;
    templateName: string;
  };
  createdAt: string;
  updatedAt: string;
}

interface ActivityItem {
  id: string;
  type: string;
  title: string;
  description: string;
  actor: { id: string; name: string | null; email: string } | null;
  proofId?: string;
  proofTitle?: string;
  createdAt: string;
}

interface DashboardData {
  stats: DashboardStats;
  waitingOnMe: ProofListItem[];
  overdue: ProofListItem[];
  needsChanges: ProofListItem[];
  recentlyApproved: ProofListItem[];
  activity: ActivityItem[];
}

export default function DashboardPage() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchDashboard();
  }, []);

  async function fetchDashboard() {
    try {
      setLoading(true);
      const res = await fetch('/api/dashboard');
      if (!res.ok) throw new Error('Failed to fetch dashboard');
      const dashboardData = await res.json();
      setData(dashboardData);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load dashboard');
    } finally {
      setLoading(false);
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="w-8 h-8 text-pyrax-500 animate-spin" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <AlertTriangle className="w-12 h-12 text-red-400" />
        <p className="text-stone-400">{error || 'Failed to load dashboard'}</p>
        <button
          onClick={fetchDashboard}
          className="flex items-center gap-2 px-4 py-2 bg-stone-800 hover:bg-stone-700 rounded-lg text-stone-200"
        >
          <RefreshCw className="w-4 h-4" />
          Retry
        </button>
      </div>
    );
  }

  const { stats, waitingOnMe, overdue, needsChanges, activity } = data;

  const statCards = [
    {
      title: 'Waiting on Me',
      value: stats.waitingOnMe,
      icon: Clock,
      color: 'text-pyrax-500',
      bgColor: 'bg-pyrax-500/10',
      href: '#waiting',
    },
    {
      title: 'Overdue',
      value: stats.overdue,
      icon: AlertTriangle,
      color: 'text-red-400',
      bgColor: 'bg-red-400/10',
      href: '#overdue',
    },
    {
      title: 'Needs Changes',
      value: stats.needsChanges,
      icon: FileText,
      color: 'text-yellow-400',
      bgColor: 'bg-yellow-400/10',
      href: '#changes',
    },
    {
      title: 'In Review',
      value: stats.inReview,
      icon: GitBranch,
      color: 'text-blue-400',
      bgColor: 'bg-blue-400/10',
      href: '/app/folders',
    },
  ];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Draft': return 'bg-stone-500';
      case 'InReview': return 'bg-blue-500';
      case 'Approved': return 'bg-green-500';
      case 'Rejected': return 'bg-red-500';
      default: return 'bg-stone-500';
    }
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'proof_created': return FileText;
      case 'version_uploaded': return FileText;
      case 'comment_added': return MessageSquare;
      case 'decision_made': return CheckCircle;
      case 'workflow_completed': return GitBranch;
      default: return FileText;
    }
  };

  const formatTimeAgo = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-stone-50">Dashboard</h1>
          <p className="text-stone-400 mt-1">
            Overview of your proofing activity
          </p>
        </div>
        <button
          onClick={fetchDashboard}
          className="flex items-center gap-2 px-3 py-2 text-stone-400 hover:text-stone-200 hover:bg-stone-800 rounded-lg transition-colors"
        >
          <RefreshCw className="w-4 h-4" />
          Refresh
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {statCards.map((stat) => (
          <Link
            key={stat.title}
            href={stat.href}
            className="bg-stone-900 rounded-xl p-6 border border-stone-800 hover:border-stone-700 transition-colors group"
          >
            <div className="flex items-center justify-between mb-4">
              <div className={`p-3 rounded-lg ${stat.bgColor}`}>
                <stat.icon className={`w-6 h-6 ${stat.color}`} />
              </div>
              <ArrowRight className="w-5 h-5 text-stone-600 group-hover:text-stone-400 transition-colors" />
            </div>
            <p className="text-3xl font-bold text-stone-50">{stat.value}</p>
            <p className="text-sm text-stone-400 mt-1">{stat.title}</p>
          </Link>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          {/* Waiting on Me */}
          <div id="waiting" className="bg-stone-900 rounded-xl border border-stone-800">
            <div className="p-4 border-b border-stone-800 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Clock className="w-5 h-5 text-pyrax-500" />
                <h2 className="font-semibold text-stone-100">Waiting on Me</h2>
                {waitingOnMe.length > 0 && (
                  <span className="px-2 py-0.5 bg-pyrax-500/20 text-pyrax-400 text-xs rounded-full">
                    {waitingOnMe.length}
                  </span>
                )}
              </div>
            </div>
            <div className="divide-y divide-stone-800">
              {waitingOnMe.length > 0 ? (
                waitingOnMe.map((proof) => (
                  <ProofListRow key={proof.id} proof={proof} />
                ))
              ) : (
                <EmptyState message="No items waiting for your review" icon={CheckCircle} />
              )}
            </div>
          </div>

          {/* Overdue */}
          <div id="overdue" className="bg-stone-900 rounded-xl border border-stone-800">
            <div className="p-4 border-b border-stone-800 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <AlertTriangle className="w-5 h-5 text-red-400" />
                <h2 className="font-semibold text-stone-100">Overdue</h2>
                {overdue.length > 0 && (
                  <span className="px-2 py-0.5 bg-red-500/20 text-red-400 text-xs rounded-full">
                    {overdue.length}
                  </span>
                )}
              </div>
            </div>
            <div className="divide-y divide-stone-800">
              {overdue.length > 0 ? (
                overdue.map((proof) => (
                  <ProofListRow key={proof.id} proof={proof} showOverdue />
                ))
              ) : (
                <EmptyState message="No overdue items" icon={CheckCircle} />
              )}
            </div>
          </div>

          {/* Needs Changes */}
          <div id="changes" className="bg-stone-900 rounded-xl border border-stone-800">
            <div className="p-4 border-b border-stone-800 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <FileText className="w-5 h-5 text-yellow-400" />
                <h2 className="font-semibold text-stone-100">Needs Changes</h2>
                {needsChanges.length > 0 && (
                  <span className="px-2 py-0.5 bg-yellow-500/20 text-yellow-400 text-xs rounded-full">
                    {needsChanges.length}
                  </span>
                )}
              </div>
            </div>
            <div className="divide-y divide-stone-800">
              {needsChanges.length > 0 ? (
                needsChanges.map((proof) => (
                  <ProofListRow key={proof.id} proof={proof} />
                ))
              ) : (
                <EmptyState message="No items need changes" icon={CheckCircle} />
              )}
            </div>
          </div>
        </div>

        {/* Right Column */}
        <div className="space-y-6">
          {/* Quick Actions */}
          <div className="bg-stone-900 rounded-xl border border-stone-800">
            <div className="p-4 border-b border-stone-800">
              <h2 className="font-semibold text-stone-100">Quick Actions</h2>
            </div>
            <div className="p-3 space-y-1">
              <Link
                href="/app/folders"
                className="flex items-center gap-3 p-3 rounded-lg hover:bg-stone-800 transition-colors"
              >
                <div className="w-9 h-9 bg-pyrax-500/10 rounded-lg flex items-center justify-center">
                  <FolderOpen className="w-5 h-5 text-pyrax-500" />
                </div>
                <div>
                  <p className="text-sm font-medium text-stone-100">Browse Folders</p>
                  <p className="text-xs text-stone-500">View and upload proofs</p>
                </div>
              </Link>
              <Link
                href="/app/department/workflows"
                className="flex items-center gap-3 p-3 rounded-lg hover:bg-stone-800 transition-colors"
              >
                <div className="w-9 h-9 bg-blue-500/10 rounded-lg flex items-center justify-center">
                  <GitBranch className="w-5 h-5 text-blue-400" />
                </div>
                <div>
                  <p className="text-sm font-medium text-stone-100">Workflow Templates</p>
                  <p className="text-xs text-stone-500">Manage approval flows</p>
                </div>
              </Link>
              <Link
                href="/app/search"
                className="flex items-center gap-3 p-3 rounded-lg hover:bg-stone-800 transition-colors"
              >
                <div className="w-9 h-9 bg-stone-700 rounded-lg flex items-center justify-center">
                  <TrendingUp className="w-5 h-5 text-stone-400" />
                </div>
                <div>
                  <p className="text-sm font-medium text-stone-100">Search</p>
                  <p className="text-xs text-stone-500">Find proofs and comments</p>
                </div>
              </Link>
            </div>
          </div>

          {/* Recent Activity */}
          <div className="bg-stone-900 rounded-xl border border-stone-800">
            <div className="p-4 border-b border-stone-800">
              <h2 className="font-semibold text-stone-100">Recent Activity</h2>
            </div>
            <div className="divide-y divide-stone-800 max-h-[400px] overflow-y-auto">
              {activity.length > 0 ? (
                activity.map((item) => {
                  const Icon = getActivityIcon(item.type);
                  return (
                    <div key={item.id} className="p-3 hover:bg-stone-800/50">
                      <div className="flex items-start gap-3">
                        <div className="w-8 h-8 bg-stone-800 rounded-full flex items-center justify-center flex-shrink-0">
                          <Icon className="w-4 h-4 text-stone-400" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <p className="text-sm text-stone-200">{item.title}</p>
                          <p className="text-xs text-stone-500 truncate">{item.description}</p>
                          <div className="flex items-center gap-2 mt-1">
                            {item.actor && (
                              <span className="text-xs text-stone-500">
                                {item.actor.name || item.actor.email}
                              </span>
                            )}
                            <span className="text-xs text-stone-600">
                              {formatTimeAgo(item.createdAt)}
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })
              ) : (
                <div className="p-6 text-center text-stone-500 text-sm">
                  No recent activity
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ProofListRow({ proof, showOverdue }: { proof: ProofListItem; showOverdue?: boolean }) {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Draft': return 'bg-stone-500';
      case 'InReview': return 'bg-blue-500';
      case 'Approved': return 'bg-green-500';
      case 'Rejected': return 'bg-red-500';
      default: return 'bg-stone-500';
    }
  };

  const daysAgo = Math.floor((Date.now() - new Date(proof.updatedAt).getTime()) / (1000 * 60 * 60 * 24));

  return (
    <Link
      href={proof.latestVersion ? `/app/viewer/${proof.latestVersion.id}` : `/app/proofs/${proof.id}`}
      className="flex items-center gap-4 p-4 hover:bg-stone-800/50 transition-colors"
    >
      <div className="w-10 h-10 bg-stone-800 rounded-lg flex items-center justify-center flex-shrink-0">
        <FileText className="w-5 h-5 text-stone-400" />
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-stone-100 truncate">{proof.title}</p>
        <div className="flex items-center gap-2 mt-0.5">
          <span className="text-xs text-stone-500">{proof.folder.name}</span>
          {proof.workflow && (
            <>
              <span className="text-stone-600">•</span>
              <span className="text-xs text-stone-500">{proof.workflow.currentStepName}</span>
            </>
          )}
        </div>
      </div>
      <div className="flex items-center gap-2">
        {showOverdue && daysAgo > 0 && (
          <span className="text-xs text-red-400">{daysAgo}d overdue</span>
        )}
        <span className={`px-2 py-0.5 text-xs font-medium rounded-full text-white ${getStatusColor(proof.status)}`}>
          {proof.status}
        </span>
      </div>
    </Link>
  );
}

function EmptyState({ message, icon: Icon }: { message: string; icon: React.ComponentType<{ className?: string }> }) {
  return (
    <div className="p-8 text-center">
      <Icon className="w-10 h-10 text-stone-700 mx-auto mb-2" />
      <p className="text-stone-500 text-sm">{message}</p>
    </div>
  );
}
