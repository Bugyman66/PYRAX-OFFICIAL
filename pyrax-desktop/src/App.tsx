import { Routes, Route } from 'react-router-dom';
import { useEffect } from 'react';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Wallet from './pages/Wallet';
import Mining from './pages/Mining';
import Explorer from './pages/Explorer';
import Settings from './pages/Settings';
import { ErrorBoundary } from './components/ErrorBoundary';
import { useNodeStore } from './stores/nodeStore';

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
    <ErrorBoundary>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="wallet" element={<Wallet />} />
          <Route path="mining" element={<Mining />} />
          <Route path="explorer" element={<Explorer />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </ErrorBoundary>
  );
}
