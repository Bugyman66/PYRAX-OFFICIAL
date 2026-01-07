'use client'

import { useState } from 'react'
import Image from 'next/image'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import {
  Dialog,
  DialogBackdrop,
  DialogPanel,
  TransitionChild,
} from '@headlessui/react'
import {
  Bars3Icon,
  XMarkIcon,
  HomeIcon,
  CubeIcon,
  ArrowsRightLeftIcon,
  CircleStackIcon,
  ChartBarIcon,
  ChevronDownIcon,
  DocumentTextIcon,
  CodeBracketIcon,
  ShieldCheckIcon,
  SparklesIcon,
  PhotoIcon,
  PresentationChartLineIcon,
  ServerStackIcon,
  GlobeAltIcon,
  BeakerIcon,
  ShareIcon,
  CommandLineIcon,
} from '@heroicons/react/24/outline'
import NetworkDropdown from './NetworkDropdown'
import NetworkStatus from './NetworkStatus'
import FaucetModal from './FaucetModal'
import SearchAutocomplete from './SearchAutocomplete'
import { isMainnet } from '@/lib/config'

interface NavItem {
  name: string
  href: string
  icon: React.ComponentType<{ className?: string }>
}

interface NavCategory {
  name: string
  icon: React.ComponentType<{ className?: string }>
  children: NavItem[]
}

const simpleNavigation: NavItem[] = [
  { name: 'Dashboard', href: '/', icon: HomeIcon },
  { name: 'Blocks', href: '/blocks', icon: CubeIcon },
  { name: 'Transactions', href: '/transactions', icon: ArrowsRightLeftIcon },
]

const categorizedNavigation: NavCategory[] = [
  {
    name: 'Contracts',
    icon: CodeBracketIcon,
    children: [
      { name: 'EVM Contracts', href: '/contracts/evm', icon: DocumentTextIcon },
      { name: 'Rust/WASM Contracts', href: '/contracts/wasm', icon: CommandLineIcon },
      { name: 'EVM Verification', href: '/contracts/evm/verify', icon: ShieldCheckIcon },
      { name: 'Rust/WASM Verification', href: '/contracts/wasm/verify', icon: ShieldCheckIcon },
    ],
  },
  {
    name: 'Tokens',
    icon: CircleStackIcon,
    children: [
      { name: 'Top Tokens', href: '/tokens', icon: SparklesIcon },
      { name: 'Token Transfers', href: '/tokens/transfers', icon: ArrowsRightLeftIcon },
    ],
  },
  {
    name: 'NFTs',
    icon: PhotoIcon,
    children: [
      { name: 'Latest Transfers', href: '/nfts/transfers', icon: ArrowsRightLeftIcon },
      { name: 'Latest Mints', href: '/nfts/mints', icon: SparklesIcon },
    ],
  },
  {
    name: 'Chain Statistics',
    icon: ChartBarIcon,
    children: [
      { name: 'Overview', href: '/stats', icon: PresentationChartLineIcon },
      { name: 'Blockchain Data', href: '/stats/blockchain', icon: CubeIcon },
      { name: 'Contract Data', href: '/stats/contracts', icon: CodeBracketIcon },
      { name: 'Network Data', href: '/stats/network', icon: GlobeAltIcon },
    ],
  },
]

const bottomNavigation: NavItem[] = [
  { name: 'Node Visualizer', href: '/visualizer/nodes', icon: ServerStackIcon },
  { name: 'Chain Visualizer', href: '/visualizer/chain', icon: ShareIcon },
]

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(' ')
}

function CollapsibleCategory({ 
  category, 
  isCurrentPath, 
  onLinkClick 
}: { 
  category: NavCategory
  isCurrentPath: (href: string) => boolean
  onLinkClick?: () => void
}) {
  const isActive = category.children.some(child => isCurrentPath(child.href))
  const [isOpen, setIsOpen] = useState(isActive)

  return (
    <li>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={classNames(
          isActive ? 'text-pyrax-500' : 'text-stone-400 hover:text-white',
          'group flex w-full items-center gap-x-3 rounded-md p-2 text-sm font-semibold hover:bg-white/5'
        )}
      >
        <category.icon className={classNames(
          isActive ? 'text-pyrax-500' : 'text-stone-500 group-hover:text-white',
          'size-5 shrink-0'
        )} />
        {category.name}
        <ChevronDownIcon className={classNames(
          isOpen ? 'rotate-180' : '',
          isActive ? 'text-pyrax-500' : 'text-stone-500',
          'ml-auto size-4 shrink-0 transition-transform duration-200'
        )} />
      </button>
      {isOpen && (
        <ul className="mt-1 space-y-1 pl-9">
          {category.children.map((child) => (
            <li key={child.name}>
              <Link
                href={child.href}
                onClick={onLinkClick}
                className={classNames(
                  isCurrentPath(child.href) ? 'text-pyrax-500' : 'text-stone-500 hover:text-white',
                  'block rounded-md py-1.5 px-2 text-sm transition-colors'
                )}
              >
                {child.name}
              </Link>
            </li>
          ))}
        </ul>
      )}
    </li>
  )
}

export default function Sidebar({ children }: { children: React.ReactNode }) {
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [faucetOpen, setFaucetOpen] = useState(false)
  const pathname = usePathname()

  const isCurrentPath = (href: string) => {
    if (href === '/') return pathname === '/'
    return pathname.startsWith(href)
  }

  return (
    <>
      <FaucetModal isOpen={faucetOpen} onClose={() => setFaucetOpen(false)} />
      <div>
        {/* Mobile sidebar */}
        <Dialog open={sidebarOpen} onClose={setSidebarOpen} className="relative z-50 lg:hidden">
          <DialogBackdrop
            transition
            className="fixed inset-0 bg-stone-950/80 transition-opacity duration-300 ease-linear data-closed:opacity-0"
          />

          <div className="fixed inset-0 flex">
            <DialogPanel
              transition
              className="relative mr-16 flex w-full max-w-xs flex-1 transform transition duration-300 ease-in-out data-closed:-translate-x-full"
            >
              <TransitionChild>
                <div className="absolute top-0 left-full flex w-16 justify-center pt-5 duration-300 ease-in-out data-closed:opacity-0">
                  <button type="button" onClick={() => setSidebarOpen(false)} className="-m-2.5 p-2.5">
                    <span className="sr-only">Close sidebar</span>
                    <XMarkIcon aria-hidden="true" className="size-6 text-white" />
                  </button>
                </div>
              </TransitionChild>

              {/* Mobile Sidebar content */}
              <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-stone-950 px-6 pb-4 ring-1 ring-white/10">
                <div className="flex h-24 shrink-0 items-center justify-center py-4">
                  <Image
                    alt="PYRAX"
                    src="/pyrax-logo.svg"
                    width={200}
                    height={200}
                    className="w-[65%] h-auto"
                  />
                </div>
                <nav className="flex flex-1 flex-col">
                  <ul role="list" className="-mx-2 space-y-1">
                    {/* Simple navigation items */}
                    {simpleNavigation.map((item) => (
                      <li key={item.name}>
                        <Link
                          href={item.href}
                          onClick={() => setSidebarOpen(false)}
                          className={classNames(
                            isCurrentPath(item.href)
                              ? 'bg-pyrax-500/10 text-pyrax-500'
                              : 'text-stone-400 hover:bg-white/5 hover:text-white',
                            'group flex gap-x-3 rounded-md p-2 text-sm font-semibold',
                          )}
                        >
                          <item.icon
                            aria-hidden="true"
                            className={classNames(
                              isCurrentPath(item.href) ? 'text-pyrax-500' : 'text-stone-500 group-hover:text-white',
                              'size-5 shrink-0',
                            )}
                          />
                          {item.name}
                        </Link>
                      </li>
                    ))}

                    {/* Collapsible categories */}
                    {categorizedNavigation.map((category) => (
                      <CollapsibleCategory
                        key={category.name}
                        category={category}
                        isCurrentPath={isCurrentPath}
                        onLinkClick={() => setSidebarOpen(false)}
                      />
                    ))}

                    {/* Divider */}
                    <li className="my-2"><div className="border-t border-stone-800" /></li>

                    {/* Bottom navigation */}
                    {bottomNavigation.map((item) => (
                      <li key={item.name}>
                        <Link
                          href={item.href}
                          onClick={() => setSidebarOpen(false)}
                          className={classNames(
                            isCurrentPath(item.href)
                              ? 'bg-pyrax-500/10 text-pyrax-500'
                              : 'text-stone-400 hover:bg-white/5 hover:text-white',
                            'group flex gap-x-3 rounded-md p-2 text-sm font-semibold',
                          )}
                        >
                          <item.icon className="size-5 shrink-0 text-stone-500 group-hover:text-white" />
                          {item.name}
                        </Link>
                      </li>
                    ))}

                  </ul>

                  {/* Network Dropdown */}
                  <div className="mt-4">
                    <div className="text-xs font-semibold text-stone-500 uppercase tracking-wider mb-2">Network</div>
                    <NetworkDropdown />
                  </div>
                </nav>
              </div>
            </DialogPanel>
          </div>
        </Dialog>

        {/* Desktop sidebar */}
        <div className="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-72 lg:flex-col">
          <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-stone-950 border-r border-stone-800 px-6 pb-4">
            <div className="flex h-24 shrink-0 items-center justify-center py-4">
              <Image
                alt="PYRAX"
                src="/pyrax-logo.svg"
                width={200}
                height={200}
                className="w-[65%] h-auto"
              />
            </div>
            <nav className="flex flex-1 flex-col">
              <ul role="list" className="-mx-2 space-y-1">
                {/* Simple navigation items */}
                {simpleNavigation.map((item) => (
                  <li key={item.name}>
                    <Link
                      href={item.href}
                      className={classNames(
                        isCurrentPath(item.href)
                          ? 'bg-pyrax-500/10 text-pyrax-500'
                          : 'text-stone-400 hover:bg-white/5 hover:text-white',
                        'group flex gap-x-3 rounded-md p-2 text-sm font-semibold transition-colors',
                      )}
                    >
                      <item.icon
                        aria-hidden="true"
                        className={classNames(
                          isCurrentPath(item.href) ? 'text-pyrax-500' : 'text-stone-500 group-hover:text-white',
                          'size-5 shrink-0 transition-colors',
                        )}
                      />
                      {item.name}
                    </Link>
                  </li>
                ))}

                {/* Collapsible categories */}
                {categorizedNavigation.map((category) => (
                  <CollapsibleCategory
                    key={category.name}
                    category={category}
                    isCurrentPath={isCurrentPath}
                  />
                ))}

                {/* Divider */}
                <li className="my-2"><div className="border-t border-stone-800" /></li>

                {/* Bottom navigation */}
                {bottomNavigation.map((item) => (
                  <li key={item.name}>
                    <Link
                      href={item.href}
                      className={classNames(
                        isCurrentPath(item.href)
                          ? 'bg-pyrax-500/10 text-pyrax-500'
                          : 'text-stone-400 hover:bg-white/5 hover:text-white',
                        'group flex gap-x-3 rounded-md p-2 text-sm font-semibold transition-colors',
                      )}
                    >
                      <item.icon className="size-5 shrink-0 text-stone-500 group-hover:text-white transition-colors" />
                      {item.name}
                    </Link>
                  </li>
                ))}

              </ul>

              {/* Network Dropdown */}
              <div className="mt-4">
                <div className="text-xs font-semibold text-stone-500 uppercase tracking-wider mb-2">Network</div>
                <NetworkDropdown />
              </div>
            </nav>
          </div>
        </div>

        {/* Main content area */}
        <div className="lg:pl-72">
          {/* Top header bar */}
          <div className="sticky top-0 z-40 flex h-16 shrink-0 items-center gap-x-4 border-b border-stone-800 bg-stone-950/80 backdrop-blur-xl px-4 sm:gap-x-6 sm:px-6 lg:px-8">
            <button
              type="button"
              onClick={() => setSidebarOpen(true)}
              className="-m-2.5 p-2.5 text-stone-400 hover:text-white lg:hidden"
            >
              <span className="sr-only">Open sidebar</span>
              <Bars3Icon aria-hidden="true" className="size-6" />
            </button>

            {/* Separator */}
            <div aria-hidden="true" className="h-6 w-px bg-stone-700 lg:hidden" />

            <div className="flex flex-1 gap-x-4 self-stretch lg:gap-x-6">
              {/* Autocomplete Search */}
              <SearchAutocomplete />
              
              <div className="flex items-center gap-x-4 lg:gap-x-6">
                {/* Faucet button - only on devnet/testnet */}
                {!isMainnet() && (
                  <button
                    type="button"
                    onClick={() => setFaucetOpen(true)}
                    className="flex items-center gap-2 rounded-lg pyrax-gradient px-3 py-1.5 text-sm font-semibold text-white hover:opacity-90 transition-opacity"
                  >
                    <BeakerIcon className="size-4" />
                    <span className="hidden sm:inline">Faucet</span>
                  </button>
                )}

                {/* Separator */}
                <div aria-hidden="true" className="hidden lg:block lg:h-6 lg:w-px lg:bg-stone-700" />

                {/* Network status indicator */}
                <NetworkStatus />
              </div>
            </div>
          </div>

          {/* Page content */}
          <main className="min-h-[calc(100vh-4rem)] bg-stone-950">
            {children}
          </main>
        </div>
      </div>
    </>
  )
}
