'use client';

import { useState, useEffect } from 'react';
import {
  Bell,
  Loader2,
  Save,
  CheckCircle,
  AlertCircle,
  AtSign,
  GitBranch,
  MessageSquare,
  FileText,
  Upload,
} from 'lucide-react';

interface NotificationPrefs {
  onAssignment: boolean;
  onMention: boolean;
  onStepChange: boolean;
  onDecision: boolean;
  onApproval: boolean;
  onSubmission: boolean;
}

const NOTIFICATION_OPTIONS = [
  {
    key: 'onAssignment',
    label: 'Step Assignments',
    description: 'Get notified when you are assigned to review a proof',
    icon: GitBranch,
  },
  {
    key: 'onMention',
    label: 'Mentions & Comments',
    description: 'Get notified when someone mentions you in a comment',
    icon: AtSign,
  },
  {
    key: 'onStepChange',
    label: 'Workflow Updates',
    description: 'Get notified when a workflow you created advances or completes',
    icon: FileText,
  },
  {
    key: 'onDecision',
    label: 'Decisions',
    description: 'Get notified when someone makes a decision on your proofs',
    icon: CheckCircle,
  },
  {
    key: 'onApproval',
    label: 'Approvals',
    description: 'Get notified when your proofs are approved',
    icon: CheckCircle,
  },
  {
    key: 'onSubmission',
    label: 'External Submissions',
    description: 'Get notified about new external submissions',
    icon: Upload,
  },
] as const;

export default function NotificationSettingsPage() {
  const [prefs, setPrefs] = useState<NotificationPrefs | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    fetchPrefs();
  }, []);

  async function fetchPrefs() {
    try {
      const res = await fetch('/api/notifications/preferences');
      if (!res.ok) throw new Error('Failed to fetch preferences');
      const data = await res.json();
      setPrefs(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load preferences');
    } finally {
      setLoading(false);
    }
  }

  async function savePrefs() {
    if (!prefs) return;

    try {
      setSaving(true);
      setError(null);
      setSaved(false);

      const res = await fetch('/api/notifications/preferences', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(prefs),
      });

      if (!res.ok) throw new Error('Failed to save preferences');
      
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save preferences');
    } finally {
      setSaving(false);
    }
  }

  function togglePref(key: keyof NotificationPrefs) {
    if (!prefs) return;
    setPrefs({ ...prefs, [key]: !prefs[key] });
  }

  function toggleAll(enabled: boolean) {
    if (!prefs) return;
    const newPrefs: NotificationPrefs = {
      onAssignment: enabled,
      onMention: enabled,
      onStepChange: enabled,
      onDecision: enabled,
      onApproval: enabled,
      onSubmission: enabled,
    };
    setPrefs(newPrefs);
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="w-8 h-8 text-pyrax-500 animate-spin" />
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto">
      <div className="flex items-center gap-4 mb-8">
        <div className="w-12 h-12 bg-pyrax-500/20 rounded-xl flex items-center justify-center">
          <Bell className="w-6 h-6 text-pyrax-500" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-stone-50">Notification Settings</h1>
          <p className="text-stone-400">Manage how you receive email notifications</p>
        </div>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4 mb-6 flex items-center gap-3">
          <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0" />
          <span className="text-red-400">{error}</span>
        </div>
      )}

      {saved && (
        <div className="bg-green-500/10 border border-green-500/30 rounded-xl p-4 mb-6 flex items-center gap-3">
          <CheckCircle className="w-5 h-5 text-green-400 flex-shrink-0" />
          <span className="text-green-400">Settings saved successfully!</span>
        </div>
      )}

      <div className="bg-stone-900 border border-stone-800 rounded-xl overflow-hidden">
        <div className="p-4 border-b border-stone-800 flex items-center justify-between">
          <h2 className="font-semibold text-stone-100">Email Notifications</h2>
          <div className="flex items-center gap-2">
            <button
              onClick={() => toggleAll(true)}
              className="text-xs text-pyrax-400 hover:text-pyrax-300 px-2 py-1"
            >
              Enable all
            </button>
            <span className="text-stone-600">|</span>
            <button
              onClick={() => toggleAll(false)}
              className="text-xs text-stone-400 hover:text-stone-300 px-2 py-1"
            >
              Disable all
            </button>
          </div>
        </div>

        {prefs && (
          <div className="divide-y divide-stone-800">
            {NOTIFICATION_OPTIONS.map((option) => {
              const Icon = option.icon;
              const isEnabled = prefs[option.key as keyof NotificationPrefs];
              
              return (
                <div
                  key={option.key}
                  className="p-4 flex items-center justify-between hover:bg-stone-800/50 transition-colors"
                >
                  <div className="flex items-center gap-4">
                    <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                      isEnabled ? 'bg-pyrax-500/20' : 'bg-stone-800'
                    }`}>
                      <Icon className={`w-5 h-5 ${isEnabled ? 'text-pyrax-400' : 'text-stone-500'}`} />
                    </div>
                    <div>
                      <p className="font-medium text-stone-100">{option.label}</p>
                      <p className="text-sm text-stone-500">{option.description}</p>
                    </div>
                  </div>
                  <button
                    onClick={() => togglePref(option.key as keyof NotificationPrefs)}
                    className={`relative w-12 h-6 rounded-full transition-colors ${
                      isEnabled ? 'bg-pyrax-500' : 'bg-stone-700'
                    }`}
                  >
                    <span
                      className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-transform ${
                        isEnabled ? 'left-7' : 'left-1'
                      }`}
                    />
                  </button>
                </div>
              );
            })}
          </div>
        )}

        <div className="p-4 border-t border-stone-800 bg-stone-900/50">
          <button
            onClick={savePrefs}
            disabled={saving || !prefs}
            className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-pyrax-500 hover:bg-pyrax-600 disabled:opacity-50 text-white font-medium rounded-lg transition-colors"
          >
            {saving ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Save className="w-4 h-4" />
            )}
            {saving ? 'Saving...' : 'Save Preferences'}
          </button>
        </div>
      </div>

      <p className="text-xs text-stone-500 mt-4 text-center">
        Notifications are sent to your registered email address. 
        You can unsubscribe from individual emails using the link in each email.
      </p>
    </div>
  );
}
