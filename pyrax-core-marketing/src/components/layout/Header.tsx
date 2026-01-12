'use client';

import { useState } from 'react';
import { Search, Bell, Menu, X } from 'lucide-react';
import type { AuthUser } from '@/lib/auth';

interface HeaderProps {
  user: AuthUser;
}

export function Header({ user }: HeaderProps) {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  return (
    <header className="sticky top-0 z-40 bg-stone-950/80 backdrop-blur-lg border-b border-stone-800">
      <div className="flex items-center justify-between h-16 px-6">
        <div className="flex items-center gap-4 lg:hidden">
          <button
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            className="p-2 text-stone-400 hover:text-stone-50"
          >
            {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
          </button>
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 pyrax-gradient rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-sm">P</span>
            </div>
            <span className="font-bold pyrax-gradient-text">PYRAX</span>
          </div>
        </div>

        <div className="hidden md:flex flex-1 max-w-xl">
          <div className="relative w-full">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-500" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search proofs, folders, comments..."
              className="w-full pl-10 pr-4 py-2 bg-stone-900 border border-stone-700 rounded-lg text-stone-50 placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-pyrax-500 focus:border-transparent transition-all"
            />
          </div>
        </div>

        <div className="flex items-center gap-4">
          <button className="relative p-2 text-stone-400 hover:text-stone-50 transition-colors">
            <Bell className="w-5 h-5" />
            <span className="absolute top-1 right-1 w-2 h-2 bg-pyrax-500 rounded-full"></span>
          </button>

          <div className="hidden sm:flex items-center gap-3 pl-4 border-l border-stone-800">
            <div className="w-8 h-8 bg-stone-800 rounded-full flex items-center justify-center">
              <span className="text-stone-50 text-sm font-medium">
                {user.name?.[0] || user.email[0].toUpperCase()}
              </span>
            </div>
            <div className="hidden md:block">
              <p className="text-sm font-medium text-stone-50">
                {user.name || 'User'}
              </p>
              <p className="text-xs text-stone-500 capitalize">
                {user.role.replace(/([A-Z])/g, ' $1').trim()}
              </p>
            </div>
          </div>
        </div>
      </div>

      {mobileMenuOpen && (
        <div className="lg:hidden border-t border-stone-800 p-4 bg-stone-900">
          <div className="relative mb-4">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-stone-500" />
            <input
              type="text"
              placeholder="Search..."
              className="w-full pl-10 pr-4 py-2 bg-stone-950 border border-stone-700 rounded-lg text-stone-50 placeholder-stone-500"
            />
          </div>
          <nav className="space-y-1">
            <a href="/app" className="block px-3 py-2 rounded-lg text-stone-50 hover:bg-stone-800">Dashboard</a>
            <a href="/app/folders" className="block px-3 py-2 rounded-lg text-stone-400 hover:bg-stone-800">Folders</a>
            <a href="/app/reviews" className="block px-3 py-2 rounded-lg text-stone-400 hover:bg-stone-800">My Reviews</a>
          </nav>
        </div>
      )}
    </header>
  );
}
