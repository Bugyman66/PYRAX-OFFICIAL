import { useState, useEffect } from 'react';
import { Settings as SettingsIcon, Save, FolderOpen, RefreshCw, Trash2, AlertTriangle, X, HardDrive } from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';

interface AppSettings {
  network: 'testnet' | 'devnet' | 'mainnet';
  autoStartNode: boolean;
  autoStartMiner: boolean;
  minerAddress?: string;
  minerThreads: number;
  cudaDevice: number;
  openclDevice: number;
  rpcPort: number;
  p2pPort: number;
  maxPeers: number;
  theme: 'light' | 'dark' | 'system';
}

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings>({
    network: 'testnet', // Testnet is the default
    autoStartNode: false,
    autoStartMiner: false,
    minerThreads: 0,
    cudaDevice: 0,
    openclDevice: -1,
    rpcPort: 8545,
    p2pPort: 30303,
    maxPeers: 50,
    theme: 'dark',
  });
  const [dataDir, setDataDir] = useState('');
  const [loading, setLoading] = useState(false);
  const [saved, setSaved] = useState(false);
  const [showClearModal, setShowClearModal] = useState(false);
  const [clearingData, setClearingData] = useState(false);
  const [clearResult, setClearResult] = useState<{ success: boolean; message: string } | null>(null);
  const [dataSizes, setDataSizes] = useState<Record<string, string>>({});

  useEffect(() => {
    loadSettings();
    loadDataSizes();
  }, []);

  const loadSettings = async () => {
    try {
      const [loadedSettings, dir] = await Promise.all([
        invoke<AppSettings>('get_settings'),
        invoke<string>('get_data_dir'),
      ]);
      setSettings(loadedSettings);
      setDataDir(dir);
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  };

  const loadDataSizes = async () => {
    try {
      const networks = ['testnet', 'devnet', 'mainnet'];
      const sizes: Record<string, string> = {};
      for (const network of networks) {
        sizes[network] = await invoke<string>('get_local_data_size', { network });
      }
      setDataSizes(sizes);
    } catch (e) {
      console.error('Failed to load data sizes:', e);
    }
  };

  const handleClearData = async (network: string) => {
    setClearingData(true);
    setClearResult(null);
    try {
      const result = await invoke<string>('clear_local_data', { network });
      setClearResult({ success: true, message: result });
      loadDataSizes(); // Refresh sizes
    } catch (e) {
      setClearResult({ success: false, message: e as string });
    } finally {
      setClearingData(false);
    }
  };

  const saveSettings = async () => {
    setLoading(true);
    try {
      await invoke('save_settings', { settings });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      setLoading(false);
    }
  };

  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
  };

  const handleBrowseDirectory = async () => {
    try {
      const result = await invoke<string | null>('browse_directory');
      if (result) {
        setDataDir(result);
        // Also save it to settings
        await invoke('set_data_dir', { path: result });
      }
    } catch (e) {
      console.error('Failed to browse directory:', e);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Settings</h1>
        <button
          onClick={saveSettings}
          disabled={loading}
          className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors disabled:opacity-50"
        >
          {loading ? (
            <RefreshCw className="animate-spin" size={16} />
          ) : saved ? (
            '✓ Saved'
          ) : (
            <>
              <Save size={16} />
              Save Settings
            </>
          )}
        </button>
      </div>

      {/* Network Selection - PROMINENT */}
      <div className="bg-gradient-to-r from-purple-900/50 to-blue-900/50 border-2 border-purple-500 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          🌐 Network Selection
        </h2>
        <p className="text-sm text-gray-300 mb-4">
          Choose which PYRAX network to connect to. <strong>Testnet is recommended</strong> for testing.
        </p>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <NetworkCard
            name="Testnet"
            description="Primary testing network"
            chainId="7972920"
            isSelected={settings.network === 'testnet'}
            isRecommended={true}
            onSelect={() => updateSetting('network', 'testnet')}
          />
          <NetworkCard
            name="Devnet"
            description="Development network"
            chainId="79729200"
            isSelected={settings.network === 'devnet'}
            isRecommended={false}
            onSelect={() => updateSetting('network', 'devnet')}
          />
          <NetworkCard
            name="Mainnet"
            description="Production network (coming soon)"
            chainId="797292"
            isSelected={settings.network === 'mainnet'}
            isRecommended={false}
            disabled={true}
            onSelect={() => {}}
          />
        </div>
      </div>

      {/* General Settings */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <SettingsIcon size={20} />
          General
        </h2>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Auto-start Node</div>
              <div className="text-sm text-gray-400">Start the node when the app launches</div>
            </div>
            <Toggle
              checked={settings.autoStartNode}
              onChange={(v) => updateSetting('autoStartNode', v)}
            />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium">Auto-start Miner</div>
              <div className="text-sm text-gray-400">Start mining when the node is ready</div>
            </div>
            <Toggle
              checked={settings.autoStartMiner}
              onChange={(v) => updateSetting('autoStartMiner', v)}
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Chain Data Directory</label>
            <p className="text-xs text-gray-500 mb-2">
              Choose where blockchain data is stored. Requires restart to take effect.
            </p>
            <div className="flex gap-2">
              <input
                type="text"
                value={dataDir}
                onChange={(e) => setDataDir(e.target.value)}
                className="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
                placeholder="Select a directory..."
              />
              <button 
                onClick={handleBrowseDirectory}
                className="px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors flex items-center gap-2"
                title="Browse for directory"
              >
                <FolderOpen size={20} />
                Browse
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Network Settings */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Network</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">RPC Port</label>
            <input
              type="number"
              value={settings.rpcPort}
              onChange={(e) => updateSetting('rpcPort', parseInt(e.target.value) || 8545)}
              className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">P2P Port</label>
            <input
              type="number"
              value={settings.p2pPort}
              onChange={(e) => updateSetting('p2pPort', parseInt(e.target.value) || 30303)}
              className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Max Peers</label>
            <input
              type="number"
              value={settings.maxPeers}
              onChange={(e) => updateSetting('maxPeers', parseInt(e.target.value) || 50)}
              min={1}
              max={200}
              className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
            />
          </div>
        </div>
      </div>

      {/* Mining Settings */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Mining</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">CPU Threads (0 = auto)</label>
            <input
              type="number"
              value={settings.minerThreads}
              onChange={(e) => updateSetting('minerThreads', parseInt(e.target.value) || 0)}
              min={0}
              max={64}
              className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">CUDA Device</label>
            <input
              type="number"
              value={settings.cudaDevice}
              onChange={(e) => updateSetting('cudaDevice', parseInt(e.target.value))}
              min={-1}
              className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">OpenCL Device (-1 = disabled)</label>
            <input
              type="number"
              value={settings.openclDevice}
              onChange={(e) => updateSetting('openclDevice', parseInt(e.target.value))}
              min={-1}
              className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
            />
          </div>
        </div>
      </div>

      {/* Appearance */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Appearance</h2>
        <div>
          <label className="block text-sm text-gray-400 mb-2">Theme</label>
          <select
            value={settings.theme}
            onChange={(e) => updateSetting('theme', e.target.value as 'light' | 'dark' | 'system')}
            className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:border-purple-500 outline-none"
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">System</option>
          </select>
        </div>
      </div>

      {/* Data Management */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <HardDrive size={20} />
          Data Management
        </h2>
        <p className="text-sm text-gray-400 mb-4">
          Clear local blockchain data if you experience sync issues or genesis mismatches. 
          The node will sync fresh from the network on next start.
        </p>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {(['testnet', 'devnet', 'mainnet'] as const).map((network) => (
            <div key={network} className="bg-gray-700/50 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="font-medium capitalize">{network}</span>
                <span className="text-sm text-gray-400">{dataSizes[network] || 'Loading...'}</span>
              </div>
              <button
                onClick={() => {
                  setShowClearModal(true);
                  setClearResult(null);
                }}
                disabled={network === 'mainnet'}
                className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-red-600/20 hover:bg-red-600/40 text-red-400 border border-red-600/50 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Trash2 size={14} />
                Clear {network} Data
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* About */}
      <div className="bg-gray-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">About</h2>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-400">Version</span>
            <span>0.1.0</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">License</span>
            <span>MIT</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Website</span>
            <a href="https://pyrax.org" className="text-purple-400 hover:text-purple-300">pyrax.org</a>
          </div>
        </div>
      </div>

      {/* Clear Data Confirmation Modal */}
      {showClearModal && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-xl p-6 max-w-md w-full mx-4 border border-gray-700 shadow-2xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold flex items-center gap-2 text-red-400">
                <AlertTriangle size={20} />
                Clear Local Data
              </h3>
              <button 
                onClick={() => setShowClearModal(false)}
                className="p-1 hover:bg-gray-700 rounded transition-colors"
              >
                <X size={20} />
              </button>
            </div>
            
            {clearResult ? (
              <div className={`p-4 rounded-lg mb-4 ${clearResult.success ? 'bg-green-900/30 border border-green-600/50' : 'bg-red-900/30 border border-red-600/50'}`}>
                <p className={clearResult.success ? 'text-green-400' : 'text-red-400'}>
                  {clearResult.message}
                </p>
              </div>
            ) : (
              <>
                <div className="bg-yellow-900/30 border border-yellow-600/50 rounded-lg p-4 mb-4">
                  <p className="text-yellow-400 text-sm">
                    <strong>Warning:</strong> This will permanently delete all local blockchain data for the selected network. 
                    You will need to sync from scratch which may take some time.
                  </p>
                </div>
                
                <p className="text-gray-300 mb-4">
                  Select which network's data to clear:
                </p>
                
                <div className="space-y-2 mb-6">
                  {(['testnet', 'devnet'] as const).map((network) => (
                    <button
                      key={network}
                      onClick={() => handleClearData(network)}
                      disabled={clearingData}
                      className="w-full flex items-center justify-between px-4 py-3 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors disabled:opacity-50"
                    >
                      <div className="flex items-center gap-3">
                        <Trash2 size={16} className="text-red-400" />
                        <span className="capitalize font-medium">{network}</span>
                      </div>
                      <span className="text-sm text-gray-400">{dataSizes[network] || 'No data'}</span>
                    </button>
                  ))}
                </div>
              </>
            )}
            
            <div className="flex justify-end gap-2">
              <button
                onClick={() => {
                  setShowClearModal(false);
                  setClearResult(null);
                }}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
              >
                {clearResult ? 'Close' : 'Cancel'}
              </button>
            </div>
            
            {clearingData && (
              <div className="absolute inset-0 bg-gray-800/80 rounded-xl flex items-center justify-center">
                <RefreshCw size={24} className="animate-spin text-purple-400" />
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`w-12 h-6 rounded-full transition-colors ${
        checked ? 'bg-purple-600' : 'bg-gray-600'
      }`}
    >
      <div
        className={`w-5 h-5 bg-white rounded-full transition-transform ${
          checked ? 'translate-x-6' : 'translate-x-0.5'
        }`}
      />
    </button>
  );
}

function NetworkCard({ 
  name, 
  description, 
  chainId, 
  isSelected, 
  isRecommended, 
  disabled,
  onSelect 
}: {
  name: string;
  description: string;
  chainId: string;
  isSelected: boolean;
  isRecommended?: boolean;
  disabled?: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      onClick={onSelect}
      disabled={disabled}
      className={`relative p-4 rounded-xl border-2 transition-all text-left ${
        disabled 
          ? 'opacity-50 cursor-not-allowed border-gray-700 bg-gray-800'
          : isSelected
            ? 'border-purple-500 bg-purple-900/30 ring-2 ring-purple-500/50'
            : 'border-gray-600 bg-gray-800 hover:border-gray-500 hover:bg-gray-750'
      }`}
    >
      {isRecommended && (
        <span className="absolute -top-2 -right-2 px-2 py-0.5 bg-green-600 text-xs rounded-full font-semibold">
          Recommended
        </span>
      )}
      <div className="font-semibold text-lg">{name}</div>
      <div className="text-sm text-gray-400 mt-1">{description}</div>
      <div className="text-xs text-gray-500 mt-2">Chain ID: {chainId}</div>
      {isSelected && (
        <div className="absolute top-3 right-3 w-4 h-4 bg-purple-500 rounded-full flex items-center justify-center">
          <span className="text-white text-xs">✓</span>
        </div>
      )}
    </button>
  );
}
