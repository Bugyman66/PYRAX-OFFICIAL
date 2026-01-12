'use client';

import { useState, useEffect, Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import { 
  Cloud, 
  CheckCircle, 
  XCircle, 
  RefreshCw, 
  ExternalLink,
  Loader2,
  Link as LinkIcon,
  User,
  FolderOpen
} from 'lucide-react';

interface DriveStatus {
  connected: boolean;
  userEmail?: string;
  rootFolderId?: string;
  rootFolderName?: string;
  error?: string;
}

function IntegrationsContent() {
  const searchParams = useSearchParams();
  const [driveStatus, setDriveStatus] = useState<DriveStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [connecting, setConnecting] = useState(false);
  
  const successMessage = searchParams.get('success');
  const errorMessage = searchParams.get('error');

  const fetchDriveStatus = async () => {
    try {
      const response = await fetch('/api/drive/status');
      const data = await response.json();
      setDriveStatus(data);
    } catch {
      setDriveStatus({
        connected: false,
        error: 'Failed to fetch Drive status',
      });
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  useEffect(() => {
    fetchDriveStatus();
  }, []);

  const handleRefresh = () => {
    setRefreshing(true);
    fetchDriveStatus();
  };

  const handleConnect = async () => {
    setConnecting(true);
    try {
      const response = await fetch('/api/auth/google');
      const data = await response.json();
      
      if (data.authUrl) {
        window.location.href = data.authUrl;
      } else {
        alert(data.error || 'Failed to get authorization URL');
        setConnecting(false);
      }
    } catch {
      alert('Failed to initiate Google connection');
      setConnecting(false);
    }
  };

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-stone-50">Integrations</h1>
        <p className="text-stone-400 mt-1">
          Manage external service connections for the Proofing Hub.
        </p>
      </div>

      {successMessage === 'connected' && (
        <div className="flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
          <CheckCircle className="w-5 h-5 text-green-400 flex-shrink-0" />
          <p className="text-green-400">Google Drive connected successfully!</p>
        </div>
      )}

      {errorMessage && (
        <div className="flex items-center gap-3 p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
          <XCircle className="w-5 h-5 text-red-400 flex-shrink-0" />
          <p className="text-red-400">Error: {decodeURIComponent(errorMessage)}</p>
        </div>
      )}

      <div className="bg-stone-900 rounded-xl border border-stone-800">
        <div className="p-6 border-b border-stone-800">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 bg-blue-500/10 rounded-xl flex items-center justify-center">
                <Cloud className="w-6 h-6 text-blue-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-stone-50">Google Drive</h2>
                <p className="text-sm text-stone-400">File storage backend</p>
              </div>
            </div>
            <button
              onClick={handleRefresh}
              disabled={refreshing}
              className="p-2 text-stone-400 hover:text-stone-50 hover:bg-stone-800 rounded-lg transition-colors disabled:opacity-50"
            >
              <RefreshCw className={`w-5 h-5 ${refreshing ? 'animate-spin' : ''}`} />
            </button>
          </div>
        </div>

        <div className="p-6">
          {loading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-8 h-8 text-stone-500 animate-spin" />
            </div>
          ) : driveStatus?.connected ? (
            <div className="space-y-6">
              <div className="flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                <CheckCircle className="w-5 h-5 text-green-400 flex-shrink-0" />
                <div>
                  <p className="text-green-400 font-medium">Connected</p>
                  <p className="text-sm text-green-400/70">
                    Google Drive is properly configured and accessible.
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="p-4 bg-stone-950 rounded-lg border border-stone-800">
                  <div className="flex items-center gap-2 text-stone-400 text-sm mb-2">
                    <User className="w-4 h-4" />
                    Connected Account
                  </div>
                  <p className="text-stone-50 font-medium truncate">
                    {driveStatus.userEmail}
                  </p>
                </div>

                <div className="p-4 bg-stone-950 rounded-lg border border-stone-800">
                  <div className="flex items-center gap-2 text-stone-400 text-sm mb-2">
                    <FolderOpen className="w-4 h-4" />
                    Root Folder
                  </div>
                  <p className="text-stone-50 font-medium">
                    {driveStatus.rootFolderName || 'PYRAX Proofing Hub'}
                  </p>
                  <p className="text-stone-500 font-mono text-xs mt-1 truncate">
                    {driveStatus.rootFolderId}
                  </p>
                </div>
              </div>

              <div className="p-4 bg-stone-950 rounded-lg border border-stone-800">
                <h3 className="text-stone-50 font-medium mb-3">Quick Actions</h3>
                <div className="flex flex-wrap gap-3">
                  <a
                    href={`https://drive.google.com/drive/folders/${driveStatus.rootFolderId}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-2 px-4 py-2 bg-stone-800 hover:bg-stone-700 text-stone-50 rounded-lg transition-colors text-sm"
                  >
                    <ExternalLink className="w-4 h-4" />
                    Open in Google Drive
                  </a>
                  <button
                    onClick={handleConnect}
                    disabled={connecting}
                    className="inline-flex items-center gap-2 px-4 py-2 bg-stone-800 hover:bg-stone-700 text-stone-50 rounded-lg transition-colors text-sm disabled:opacity-50"
                  >
                    <RefreshCw className={`w-4 h-4 ${connecting ? 'animate-spin' : ''}`} />
                    Reconnect
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <div className="space-y-6">
              <div className="flex items-center gap-3 p-4 bg-amber-500/10 border border-amber-500/20 rounded-lg">
                <XCircle className="w-5 h-5 text-amber-400 flex-shrink-0" />
                <div>
                  <p className="text-amber-400 font-medium">Not Connected</p>
                  <p className="text-sm text-amber-400/70">
                    {driveStatus?.error || 'Connect your Google account to enable file storage.'}
                  </p>
                </div>
              </div>

              <div className="p-6 bg-stone-950 rounded-lg border border-stone-800 text-center">
                <Cloud className="w-12 h-12 text-stone-600 mx-auto mb-4" />
                <h3 className="text-stone-50 font-medium mb-2">Connect Google Drive</h3>
                <p className="text-sm text-stone-400 mb-6 max-w-md mx-auto">
                  Connect your Google account to store proof files securely in Google Drive.
                  A folder called &quot;PYRAX Proofing Hub&quot; will be created automatically.
                </p>
                <button
                  onClick={handleConnect}
                  disabled={connecting}
                  className="inline-flex items-center gap-2 px-6 py-3 pyrax-gradient text-white font-semibold rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50"
                >
                  {connecting ? (
                    <Loader2 className="w-5 h-5 animate-spin" />
                  ) : (
                    <LinkIcon className="w-5 h-5" />
                  )}
                  {connecting ? 'Connecting...' : 'Connect Google Drive'}
                </button>
              </div>

              <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                <h4 className="text-blue-400 font-medium mb-2">Prerequisites</h4>
                <ul className="text-sm text-blue-400/70 space-y-1">
                  <li>• OAuth credentials must be configured in environment variables</li>
                  <li>• GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET must be set</li>
                  <li>• Redirect URI must be registered in Google Cloud Console</li>
                </ul>
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="bg-stone-900 rounded-xl border border-stone-800 opacity-60">
        <div className="p-6 border-b border-stone-800">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 bg-green-500/10 rounded-xl flex items-center justify-center">
              <svg className="w-6 h-6 text-green-400" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
              </svg>
            </div>
            <div>
              <h2 className="text-lg font-semibold text-stone-50">Brevo Email</h2>
              <p className="text-sm text-stone-400">Transactional email service</p>
            </div>
          </div>
        </div>
        <div className="p-6">
          <p className="text-stone-500 text-sm">
            Email integration will be configured in Phase 10.
          </p>
        </div>
      </div>
    </div>
  );
}

export default function IntegrationsPage() {
  return (
    <Suspense fallback={
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="w-8 h-8 text-stone-500 animate-spin" />
      </div>
    }>
      <IntegrationsContent />
    </Suspense>
  );
}
