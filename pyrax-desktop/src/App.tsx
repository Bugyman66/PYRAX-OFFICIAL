import { Routes, Route } from 'react-router-dom';
import { useEffect, lazy, Suspense } from 'react';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import { ErrorBoundary } from './components/ErrorBoundary';
import ToastContainer from './components/ToastContainer';
import UpdateNotification from './components/UpdateNotification';
import { useNodeStore } from './stores/nodeStore';

// Lazy load non-critical pages for faster initial load
const Wallet = lazy(() => import('./pages/Wallet'));
const Mining = lazy(() => import('./pages/Mining'));
const Explorer = lazy(() => import('./pages/Explorer'));
const Settings = lazy(() => import('./pages/Settings'));

// Loading fallback for lazy routes
const PageLoader = () => (
  <div className="flex items-center justify-center h-full">
    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
  </div>
);

export default function App() {
  const { fetchStatus } = useNodeStore();

  useEffect(() => {
    // Poll node status every 3 seconds
    const safeF = async () => {
      try {
        await fetchStatus();
      } catch (e) {
        console.error('Status fetch error:', e);
      }
    };
    safeF();
    const interval = setInterval(safeF, 3000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  return (
    <>
      <ErrorBoundary>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Dashboard />} />
            <Route path="wallet" element={<Suspense fallback={<PageLoader />}><Wallet /></Suspense>} />
            <Route path="mining" element={<Suspense fallback={<PageLoader />}><Mining /></Suspense>} />
            <Route path="explorer" element={<Suspense fallback={<PageLoader />}><Explorer /></Suspense>} />
            <Route path="settings" element={<Suspense fallback={<PageLoader />}><Settings /></Suspense>} />
          </Route>
        </Routes>
      </ErrorBoundary>
      <ToastContainer />
      <UpdateNotification />
    </>
  );
}
