import { Outlet, NavLink } from 'react-router-dom';
import { 
  LayoutDashboard, 
  Wallet, 
  Hammer, 
  Search, 
  Settings,
  Circle,
  Loader2
} from 'lucide-react';
import { useNodeStore } from '../stores/nodeStore';
import { cn } from '../lib/utils';

const APP_VERSION = '0.2.16';

const navItems = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/wallet', icon: Wallet, label: 'Wallet' },
  { to: '/mining', icon: Hammer, label: 'Mining' },
  { to: '/explorer', icon: Search, label: 'Explorer' },
  { to: '/settings', icon: Settings, label: 'Settings' },
];

export default function Layout() {
  const { status } = useNodeStore();

  return (
    <div className="flex h-screen bg-dark-900 text-stone-100">
      {/* Sidebar */}
      <aside className="w-64 bg-dark-800 border-r border-dark-600 flex flex-col">
        {/* Logo */}
        <div className="p-4 border-b border-dark-600">
          <div className="flex items-center gap-3">
            <img src="/pyrax-logo.png" alt="PYRAX" className="w-10 h-10 object-contain" />
            <div>
              <h1 className="text-xl font-bold gradient-text">PYRAX</h1>
              <p className="text-xs text-stone-500">v{APP_VERSION}</p>
            </div>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-4 space-y-1">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 px-3 py-2 rounded-lg transition-colors',
                  isActive
                    ? 'bg-pyrax-600 text-white'
                    : 'text-stone-400 hover:bg-dark-700 hover:text-white'
                )
              }
            >
              <Icon size={20} />
              <span>{label}</span>
            </NavLink>
          ))}
        </nav>

        {/* Node Status */}
        <div className="p-4 border-t border-dark-600">
          <div className="flex items-center gap-2 text-sm">
            {status?.running ? (
              status?.connected ? (
                <>
                  <Circle className="w-3 h-3 fill-green-500 text-green-500" />
                  <span className="text-green-400">Connected</span>
                </>
              ) : (
                <>
                  <Loader2 className="w-3 h-3 text-pyrax-500 animate-spin" />
                  <span className="text-pyrax-400">Connecting...</span>
                </>
              )
            ) : (
              <>
                <Circle className="w-3 h-3 fill-red-500 text-red-500" />
                <span className="text-red-400">Node Offline</span>
              </>
            )}
          </div>
          {status?.running && status?.connected && (
            <div className="mt-2 text-xs text-stone-500">
              <div>Block: #{status.blockHeight.toLocaleString()}</div>
              <div>Peers: {status.peerCount}</div>
              <div>Network: {status.network}</div>
            </div>
          )}
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto bg-dark-900">
        <Outlet />
      </main>
    </div>
  );
}
