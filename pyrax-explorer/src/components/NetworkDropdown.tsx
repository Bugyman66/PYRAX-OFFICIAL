'use client'

import { Fragment } from 'react'
import { Menu, MenuButton, MenuItem, MenuItems, Transition } from '@headlessui/react'
import { ChevronUpDownIcon, CheckIcon } from '@heroicons/react/20/solid'
import { useNetwork } from '@/context/NetworkContext'
import { NETWORKS, Network } from '@/lib/networks'

function classNames(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(' ')
}

export default function NetworkDropdown() {
  const { networkState, setNetwork } = useNetwork()

  const handleNetworkChange = (network: Network) => {
    setNetwork(network)
  }

  return (
    <Menu as={'div' as const} className="relative">
      <MenuButton className="w-full group flex items-center gap-x-3 rounded-lg p-2 text-sm font-semibold text-stone-400 hover:bg-white/5 hover:text-white transition-colors">
        <span
          className={classNames(
            networkState.network.id === networkState.network.id
              ? 'border-pyrax-500 text-pyrax-500 bg-pyrax-500/10'
              : 'border-stone-700 text-stone-500 bg-stone-900',
            'flex size-6 shrink-0 items-center justify-center rounded-lg border text-[0.625rem] font-medium',
          )}
        >
          {networkState.network.name.charAt(0)}
        </span>
        <span className="flex-1 text-left truncate">{networkState.network.name}</span>
        <ChevronUpDownIcon className="size-5 text-stone-500 group-hover:text-white" />
      </MenuButton>

      <Transition
        as={Fragment}
        enter="transition ease-out duration-100"
        enterFrom="transform opacity-0 scale-95"
        enterTo="transform opacity-100 scale-100"
        leave="transition ease-in duration-75"
        leaveFrom="transform opacity-100 scale-100"
        leaveTo="transform opacity-0 scale-95"
      >
        <MenuItems className="absolute left-0 right-0 z-10 mt-2 origin-top rounded-xl bg-stone-900 border border-stone-700 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none overflow-hidden">
          <div className="py-1">
            {NETWORKS.map((network) => (
              <MenuItem key={network.id}>
                {({ active }: { active: boolean }) => (
                  <button
                    onClick={() => handleNetworkChange(network)}
                    className={classNames(
                      active ? 'bg-white/5' : '',
                      networkState.network.id === network.id ? 'text-pyrax-500' : 'text-stone-300',
                      'w-full flex items-center gap-3 px-3 py-2 text-sm',
                    )}
                  >
                    <span
                      className={classNames(
                        networkState.network.id === network.id
                          ? 'border-pyrax-500 text-pyrax-500 bg-pyrax-500/10'
                          : 'border-stone-700 text-stone-500 bg-stone-800',
                        'flex size-6 shrink-0 items-center justify-center rounded-lg border text-[0.625rem] font-medium',
                      )}
                    >
                      {network.name.charAt(0)}
                    </span>
                    <span className="flex-1 text-left">{network.name}</span>
                    {networkState.network.id === network.id && (
                      <CheckIcon className="size-4 text-pyrax-500" />
                    )}
                  </button>
                )}
              </MenuItem>
            ))}
          </div>
          <div className="border-t border-stone-700 px-3 py-2 space-y-1">
            <p className="text-xs text-stone-500 font-semibold">TriStream RPCs:</p>
            <p className="text-[10px] text-stone-500">
              <span className="text-stone-400">A:</span> <span className="font-mono">{networkState.network.streams.A}</span>
            </p>
            <p className="text-[10px] text-stone-500">
              <span className="text-stone-400">B:</span> <span className="font-mono">{networkState.network.streams.B}</span>
            </p>
            <p className="text-[10px] text-stone-500">
              <span className="text-stone-400">C:</span> <span className="font-mono">{networkState.network.streams.C}</span>
            </p>
          </div>
        </MenuItems>
      </Transition>
    </Menu>
  )
}
