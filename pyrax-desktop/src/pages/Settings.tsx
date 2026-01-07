import { useState, useEffect } from 'react';
import { Settings as SettingsIcon, Save, FolderOpen, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';

interface AppSettings {
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

  useEffect(() => {
    loadSettings();
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
            <label className="block text-sm text-gray-400 mb-2">Data Directory</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={dataDir}
                readOnly
                className="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-gray-400"
              />
              <button className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg">
                <FolderOpen size={20} />
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
