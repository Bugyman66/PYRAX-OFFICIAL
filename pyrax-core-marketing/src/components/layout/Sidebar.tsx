'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import {
  LayoutDashboard,
  FolderOpen,
  FileCheck,
  Settings,
  Users,
  Workflow,
  Globe,
  LogOut,
  Plug,
} from 'lucide-react';
import type { AuthUser } from '@/lib/auth';

interface SidebarProps {
  user: AuthUser;
}

export function Sidebar({ user }: SidebarProps) {
  const pathname = usePathname();

  const navigation = [
    { name: 'Dashboard', href: '/app', icon: LayoutDashboard },
    { name: 'Folders', href: '/app/folders', icon: FolderOpen },
    { name: 'My Reviews', href: '/app/reviews', icon: FileCheck },
  ];

  const departmentNav = [
    { name: 'Workflows', href: '/app/department/workflows', icon: Workflow },
    { name: 'Media Kit', href: '/app/department/media-kit', icon: Globe },
  ];

  const adminNav = [
    { name: 'Users', href: '/app/admin/users', icon: Users },
    { name: 'Integrations', href: '/app/admin/integrations', icon: Plug },
    { name: 'Settings', href: '/app/admin/settings', icon: Settings },
  ];

  const isActive = (href: string) => {
    if (href === '/app') return pathname === '/app';
    return pathname.startsWith(href);
  };

  const handleLogout = async () => {
    await fetch('/api/auth/logout', { method: 'POST' });
    window.location.href = '/auth/login';
  };

  return (
    <aside className="fixed inset-y-0 left-0 z-50 w-64 bg-stone-900 border-r border-stone-800 hidden lg:block">
      <div className="flex flex-col h-full">
        <div className="p-6 border-b border-stone-800">
          <Link href="/app" className="flex items-center gap-3">
            <div className="w-10 h-10 pyrax-gradient rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-lg">P</span>
            </div>
            <div>
              <h1 className="text-lg font-bold pyrax-gradient-text">PYRAX</h1>
              <p className="text-xs text-stone-500">Proofing Hub</p>
            </div>
          </Link>
        </div>

        <nav className="flex-1 p-4 space-y-1 overflow-y-auto">
          <div className="mb-6">
            <p className="px-3 mb-2 text-xs font-semibold text-stone-500 uppercase tracking-wider">
              Main
            </p>
            {navigation.map((item) => (
              <Link
                key={item.name}
                href={item.href}
                className={`flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                  isActive(item.href)
                    ? 'bg-pyrax-500/10 text-pyrax-500'
                    : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
                }`}
              >
                <item.icon className="w-5 h-5" />
                {item.name}
              </Link>
            ))}
          </div>

          {(user.role === 'OrgAdmin' || user.role === 'DepartmentHead') && (
            <div className="mb-6">
              <p className="px-3 mb-2 text-xs font-semibold text-stone-500 uppercase tracking-wider">
                Department
              </p>
              {departmentNav.map((item) => (
                <Link
                  key={item.name}
                  href={item.href}
                  className={`flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                    isActive(item.href)
                      ? 'bg-pyrax-500/10 text-pyrax-500'
                      : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
                  }`}
                >
                  <item.icon className="w-5 h-5" />
                  {item.name}
                </Link>
              ))}
            </div>
          )}

          {user.role === 'OrgAdmin' && (
            <div className="mb-6">
              <p className="px-3 mb-2 text-xs font-semibold text-stone-500 uppercase tracking-wider">
                Admin
              </p>
              {adminNav.map((item) => (
                <Link
                  key={item.name}
                  href={item.href}
                  className={`flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                    isActive(item.href)
                      ? 'bg-pyrax-500/10 text-pyrax-500'
                      : 'text-stone-400 hover:text-stone-50 hover:bg-stone-800'
                  }`}
                >
                  <item.icon className="w-5 h-5" />
                  {item.name}
                </Link>
              ))}
            </div>
          )}
        </nav>

        <div className="p-4 border-t border-stone-800">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 bg-stone-800 rounded-full flex items-center justify-center">
              <span className="text-stone-50 font-medium">
                {user.name?.[0] || user.email[0].toUpperCase()}
              </span>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-stone-50 truncate">
                {user.name || 'User'}
              </p>
              <p className="text-xs text-stone-500 truncate">{user.email}</p>
            </div>
          </div>
          <button
            onClick={handleLogout}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-stone-400 hover:text-stone-50 hover:bg-stone-800 transition-colors"
          >
            <LogOut className="w-5 h-5" />
            Sign out
          </button>
        </div>
      </div>
    </aside>
  );
}
