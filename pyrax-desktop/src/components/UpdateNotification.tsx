import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Download, X, RefreshCw, Sparkles, Loader2 } from 'lucide-react';

interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion: string | null;
  releaseNotes: string | null;
  downloadUrl: string | null;
}

export default function UpdateNotification() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(true); // Start true to show loading on launch
  const [initialCheck, setInitialCheck] = useState(true);
  const [installing, setInstalling] = useState(false);
  const [dismissed, setDismissed] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkForUpdates = async (isInitial = false) => {
    setChecking(true);
    setError(null);
    try {
      const info = await invoke<UpdateInfo>('check_for_updates');
      setUpdateInfo(info);
      if (info.available) {
        setDismissed(false);
      }
    } catch (e) {
      console.error('Failed to check for updates:', e);
      setError(String(e));
    } finally {
      setChecking(false);
      if (isInitial) setInitialCheck(false);
    }
  };

  const installUpdate = async () => {
    setInstalling(true);
    setError(null);
    try {
      await invoke('install_update');
    } catch (e) {
      console.error('Failed to install update:', e);
      setError(String(e));
      setInstalling(false);
    }
  };

  useEffect(() => {
    // Check for updates immediately on app launch
    checkForUpdates(true);
    
    // Check every 30 minutes
    const interval = setInterval(() => checkForUpdates(false), 30 * 60 * 1000);
    return () => clearInterval(interval);
  }, []);

  // Show checking indicator on initial app launch
  if (initialCheck && checking) {
    return (
      <div className="fixed top-4 right-4 z-[9999] animate-fade-in">
        <div className="flex items-center gap-2 px-4 py-2 bg-gray-800/90 rounded-lg border border-gray-700 shadow-lg">
          <Loader2 size={16} className="animate-spin text-purple-400" />
          <span className="text-sm text-gray-300">Checking for updates...</span>
        </div>
      </div>
    );
  }

  if (dismissed || !updateInfo?.available) {
    return null;
  }

  return (
    <div className="fixed top-4 right-4 z-[9999] max-w-sm animate-slide-in">
      <div className="bg-gradient-to-r from-purple-900 to-indigo-900 rounded-xl border border-purple-500/50 shadow-2xl shadow-purple-500/20 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 bg-black/20">
          <div className="flex items-center gap-2">
            <Sparkles className="text-yellow-400" size={18} />
            <span className="font-semibold text-white">Update Available</span>
          </div>
          <button
            onClick={() => setDismissed(true)}
            className="p-1 hover:bg-white/10 rounded transition-colors"
            title="Dismiss"
          >
            <X size={16} className="text-gray-400" />
          </button>
        </div>

        {/* Content */}
        <div className="px-4 py-3 space-y-3">
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Current:</span>
            <span className="font-mono text-gray-300">v{updateInfo.currentVersion}</span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Latest:</span>
            <span className="font-mono text-green-400">v{updateInfo.latestVersion}</span>
          </div>

          {updateInfo.releaseNotes && (
            <div className="text-xs text-gray-400 bg-black/20 rounded p-2 max-h-20 overflow-y-auto">
              {updateInfo.releaseNotes}
            </div>
          )}

          {error && (
            <div className="text-xs text-red-400 bg-red-900/20 rounded p-2">
              {error}
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2 pt-1">
            <button
              onClick={installUpdate}
              disabled={installing}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50 font-medium"
            >
              {installing ? (
                <>
                  <RefreshCw size={16} className="animate-spin" />
                  Installing...
                </>
              ) : (
                <>
                  <Download size={16} />
                  Install Update
                </>
              )}
            </button>
            <button
              onClick={() => setDismissed(true)}
              className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
            >
              Later
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
