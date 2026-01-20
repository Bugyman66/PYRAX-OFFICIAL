import { useState, useEffect } from 'react';
import { writeText } from '@tauri-apps/api/clipboard';
import { X, Copy, Check, AlertCircle, CheckCircle, AlertTriangle, Info } from 'lucide-react';
import { useToastStore, Toast } from '../stores/toastStore';

function ToastItem({ toast, onRemove }: { toast: Toast; onRemove: () => void }) {
  const [copied, setCopied] = useState(false);
  const [remainingSeconds, setRemainingSeconds] = useState(60);
  
  // Update countdown every second
  useEffect(() => {
    const updateRemaining = () => {
      const elapsed = Date.now() - toast.timestamp;
      const remaining = Math.max(0, 60000 - elapsed);
      setRemainingSeconds(Math.ceil(remaining / 1000));
    };
    
    updateRemaining();
    const interval = setInterval(updateRemaining, 1000);
    return () => clearInterval(interval);
  }, [toast.timestamp]);
  
  const handleCopy = async () => {
    try {
      // CLIPBOARD FIX: Use Tauri clipboard API
      await writeText(toast.message);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      // Fallback to navigator.clipboard
      try {
        await navigator.clipboard.writeText(toast.message);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
        return;
      } catch (_) {}
      console.error('Failed to copy:', e);
    }
  };
  
  const getIcon = () => {
    switch (toast.type) {
      case 'error':
        return <AlertCircle className="text-red-400 flex-shrink-0" size={20} />;
      case 'success':
        return <CheckCircle className="text-green-400 flex-shrink-0" size={20} />;
      case 'warning':
        return <AlertTriangle className="text-yellow-400 flex-shrink-0" size={20} />;
      case 'info':
        return <Info className="text-blue-400 flex-shrink-0" size={20} />;
    }
  };
  
  const getBorderColor = () => {
    switch (toast.type) {
      case 'error':
        return 'border-red-700 bg-red-900/90';
      case 'success':
        return 'border-green-700 bg-green-900/90';
      case 'warning':
        return 'border-yellow-700 bg-yellow-900/90';
      case 'info':
        return 'border-blue-700 bg-blue-900/90';
    }
  };
  
  // Calculate remaining for progress bar
  const elapsed = Date.now() - toast.timestamp;
  const remaining = Math.max(0, 60000 - elapsed);
  
  return (
    <div
      className={`relative flex items-start gap-3 p-4 rounded-lg border backdrop-blur-sm shadow-lg max-w-md ${getBorderColor()}`}
    >
      {getIcon()}
      
      <div className="flex-1 min-w-0">
        <p className="text-sm text-gray-100 break-words select-all cursor-text">
          {toast.message}
        </p>
        <p className="text-xs text-gray-400 mt-1">
          Auto-dismiss in {remainingSeconds}s
        </p>
      </div>
      
      <div className="flex items-center gap-1 flex-shrink-0">
        <button
          onClick={handleCopy}
          className="p-1.5 hover:bg-white/10 rounded transition-colors"
          title="Copy to clipboard"
        >
          {copied ? (
            <Check className="text-green-400" size={16} />
          ) : (
            <Copy className="text-gray-400 hover:text-white" size={16} />
          )}
        </button>
        <button
          onClick={onRemove}
          className="p-1.5 hover:bg-white/10 rounded transition-colors"
          title="Dismiss"
        >
          <X className="text-gray-400 hover:text-white" size={16} />
        </button>
      </div>
      
      {/* Progress bar showing remaining time */}
      <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/20 rounded-b-lg overflow-hidden">
        <div
          className="h-full bg-white/30 transition-all duration-1000"
          style={{ width: `${(remaining / 60000) * 100}%` }}
        />
      </div>
    </div>
  );
}

export default function ToastContainer() {
  const { toasts, removeToast } = useToastStore();
  
  console.log('[ToastContainer] Rendering with', toasts.length, 'toasts');
  
  if (toasts.length === 0) return null;
  
  return (
    <div className="fixed bottom-4 right-4 z-[9999] flex flex-col gap-2 max-h-[80vh] overflow-y-auto pointer-events-auto">
      {toasts.map((toast) => (
        <ToastItem
          key={toast.id}
          toast={toast}
          onRemove={() => removeToast(toast.id)}
        />
      ))}
    </div>
  );
}
