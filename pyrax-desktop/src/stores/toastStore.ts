import { create } from 'zustand';

export interface Toast {
  id: string;
  type: 'error' | 'success' | 'warning' | 'info';
  message: string;
  timestamp: number;
}

interface ToastStore {
  toasts: Toast[];
  addToast: (type: Toast['type'], message: string) => void;
  removeToast: (id: string) => void;
  clearAll: () => void;
}

export const useToastStore = create<ToastStore>((set, get) => ({
  toasts: [],
  
  addToast: (type, message) => {
    const id = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const toast: Toast = {
      id,
      type,
      message,
      timestamp: Date.now(),
    };
    
    set((state) => ({
      toasts: [...state.toasts, toast],
    }));
    
    // Auto-remove after 60 seconds (1 minute)
    setTimeout(() => {
      get().removeToast(id);
    }, 60000);
  },
  
  removeToast: (id) => {
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    }));
  },
  
  clearAll: () => {
    set({ toasts: [] });
  },
}));

// Helper functions for easy usage
export const toast = {
  error: (message: string) => useToastStore.getState().addToast('error', message),
  success: (message: string) => useToastStore.getState().addToast('success', message),
  warning: (message: string) => useToastStore.getState().addToast('warning', message),
  info: (message: string) => useToastStore.getState().addToast('info', message),
};
