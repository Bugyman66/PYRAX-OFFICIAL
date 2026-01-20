import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  Cpu, Thermometer, Zap, DollarSign, Clock, TrendingUp,
  Activity, AlertTriangle, Settings, Play, Pause, RefreshCw
} from 'lucide-react';

interface MiningStats {
  hashrate: number;
  hashrateUnit: string;
  shares: { accepted: number; rejected: number; stale: number };
  blocksFound: number;
  uptime: number;
  difficulty: number;
  estimatedEarnings: { hourly: number; daily: number; monthly: number };
}

interface GpuInfo {
  id: number;
  name: string;
  hashrate: number;
  temperature: number;
  fanSpeed: number;
  power: number;
  memory: { used: number; total: number };
  utilization: number;
}

interface PoolInfo {
  name: string;
  url: string;
  fee: number;
  latency: number;
  connected: boolean;
}

interface OverclockProfile {
  name: string;
  coreClock: number;
  memoryClock: number;
  powerLimit: number;
  fanSpeed: number;
}

const defaultProfiles: OverclockProfile[] = [
  { name: 'Efficiency', coreClock: -200, memoryClock: 1000, powerLimit: 70, fanSpeed: 60 },
  { name: 'Balanced', coreClock: 0, memoryClock: 800, powerLimit: 85, fanSpeed: 70 },
  { name: 'Performance', coreClock: 150, memoryClock: 1200, powerLimit: 100, fanSpeed: 80 },
];

export default function MiningDashboard() {
  const [mining, setMining] = useState(false);
  const [stats, setStats] = useState<MiningStats | null>(null);
  const [gpus, setGpus] = useState<GpuInfo[]>([]);
  const [pools, setPools] = useState<PoolInfo[]>([]);
  const [electricityRate, setElectricityRate] = useState(0.12);
  const [selectedProfile, setSelectedProfile] = useState('Balanced');
  const [schedule, setSchedule] = useState({ enabled: false, startHour: 22, endHour: 8 });
  const [alerts, setAlerts] = useState<string[]>([]);

  const fetchMiningData = useCallback(async () => {
    try {
      const data = await invoke<any>('get_mining_stats');
      setStats(data.stats);
      setGpus(data.gpus || []);
      setMining(data.mining);
    } catch (e) {
      console.error('Failed to fetch mining data:', e);
    }
  }, []);

  useEffect(() => {
    fetchMiningData();
    const interval = setInterval(fetchMiningData, 2000);
    return () => clearInterval(interval);
  }, [fetchMiningData]);

  useEffect(() => {
    // Check temperature alerts
    const hotGpus = gpus.filter(g => g.temperature > 80);
    if (hotGpus.length > 0) {
      setAlerts(prev => [...prev.slice(-4), `GPU ${hotGpus[0].id} temperature critical: ${hotGpus[0].temperature}°C`]);
    }
  }, [gpus]);

  const toggleMining = async () => {
    try {
      await invoke(mining ? 'stop_mining' : 'start_mining');
      setMining(!mining);
    } catch (e) {
      console.error('Failed to toggle mining:', e);
    }
  };

  const applyProfile = async (profile: OverclockProfile) => {
    setSelectedProfile(profile.name);
    try {
      await invoke('apply_overclock_profile', { profile });
    } catch (e) {
      console.error('Failed to apply profile:', e);
    }
  };

  const calculateProfitability = () => {
    if (!stats) return { profit: 0, powerCost: 0 };
    const totalPower = gpus.reduce((sum, g) => sum + g.power, 0);
    const powerCost = (totalPower / 1000) * 24 * electricityRate;
    const profit = stats.estimatedEarnings.daily - powerCost;
    return { profit, powerCost };
  };

  const formatHashrate = (h: number) => {
    if (h >= 1e12) return `${(h / 1e12).toFixed(2)} TH/s`;
    if (h >= 1e9) return `${(h / 1e9).toFixed(2)} GH/s`;
    if (h >= 1e6) return `${(h / 1e6).toFixed(2)} MH/s`;
    if (h >= 1e3) return `${(h / 1e3).toFixed(2)} KH/s`;
    return `${h.toFixed(2)} H/s`;
  };

  const formatUptime = (secs: number) => {
    const days = Math.floor(secs / 86400);
    const hours = Math.floor((secs % 86400) / 3600);
    const mins = Math.floor((secs % 3600) / 60);
    return days > 0 ? `${days}d ${hours}h` : `${hours}h ${mins}m`;
  };

  const { profit, powerCost } = calculateProfitability();

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white flex items-center gap-2">
          <Cpu className="w-7 h-7 text-orange-500" />
          Mining Dashboard
        </h1>
        <div className="flex items-center gap-3">
          <button onClick={fetchMiningData} className="p-2 rounded bg-gray-700 hover:bg-gray-600">
            <RefreshCw className="w-5 h-5" />
          </button>
          <button
            onClick={toggleMining}
            className={`flex items-center gap-2 px-4 py-2 rounded font-medium ${
              mining ? 'bg-red-600 hover:bg-red-700' : 'bg-green-600 hover:bg-green-700'
            }`}
          >
            {mining ? <Pause className="w-5 h-5" /> : <Play className="w-5 h-5" />}
            {mining ? 'Stop Mining' : 'Start Mining'}
          </button>
        </div>
      </div>

      {/* Alerts */}
      {alerts.length > 0 && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg p-3">
          <div className="flex items-center gap-2 text-red-400">
            <AlertTriangle className="w-5 h-5" />
            <span className="font-medium">Alerts</span>
          </div>
          {alerts.slice(-3).map((alert, i) => (
            <div key={i} className="text-sm text-red-300 mt-1">{alert}</div>
          ))}
        </div>
      )}

      {/* Stats Overview */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard
          icon={<Activity className="w-6 h-6 text-blue-500" />}
          label="Hashrate"
          value={stats ? formatHashrate(stats.hashrate) : '--'}
        />
        <StatCard
          icon={<TrendingUp className="w-6 h-6 text-green-500" />}
          label="Shares"
          value={stats ? `${stats.shares.accepted}/${stats.shares.rejected}` : '--'}
          subtext="Accepted/Rejected"
        />
        <StatCard
          icon={<Clock className="w-6 h-6 text-purple-500" />}
          label="Uptime"
          value={stats ? formatUptime(stats.uptime) : '--'}
        />
        <StatCard
          icon={<DollarSign className="w-6 h-6 text-yellow-500" />}
          label="Est. Daily"
          value={stats ? `$${stats.estimatedEarnings.daily.toFixed(2)}` : '--'}
        />
      </div>

      {/* Profitability Calculator */}
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <DollarSign className="w-5 h-5 text-yellow-500" />
          Profitability
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label className="text-sm text-gray-400">Electricity Rate ($/kWh)</label>
            <input
              type="number"
              value={electricityRate}
              onChange={(e) => setElectricityRate(parseFloat(e.target.value) || 0)}
              className="w-full mt-1 px-3 py-2 bg-gray-700 rounded border border-gray-600"
              step="0.01"
            />
          </div>
          <div className="bg-gray-700 rounded p-3">
            <div className="text-sm text-gray-400">Daily Power Cost</div>
            <div className="text-xl font-bold text-red-400">${powerCost.toFixed(2)}</div>
          </div>
          <div className="bg-gray-700 rounded p-3">
            <div className="text-sm text-gray-400">Net Daily Profit</div>
            <div className={`text-xl font-bold ${profit >= 0 ? 'text-green-400' : 'text-red-400'}`}>
              ${profit.toFixed(2)}
            </div>
          </div>
        </div>
      </div>

      {/* GPU Status */}
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Cpu className="w-5 h-5 text-blue-500" />
          GPU Status
        </h2>
        <div className="space-y-3">
          {gpus.length === 0 ? (
            <div className="text-gray-400 text-center py-4">No GPUs detected</div>
          ) : (
            gpus.map((gpu) => (
              <div key={gpu.id} className="bg-gray-700 rounded-lg p-3">
                <div className="flex items-center justify-between mb-2">
                  <span className="font-medium">GPU {gpu.id}: {gpu.name}</span>
                  <span className="text-blue-400">{formatHashrate(gpu.hashrate)}</span>
                </div>
                <div className="grid grid-cols-4 gap-2 text-sm">
                  <GpuMetric
                    icon={<Thermometer className="w-4 h-4" />}
                    label="Temp"
                    value={`${gpu.temperature}°C`}
                    warning={gpu.temperature > 80}
                  />
                  <GpuMetric
                    icon={<Zap className="w-4 h-4" />}
                    label="Power"
                    value={`${gpu.power}W`}
                  />
                  <GpuMetric
                    icon={<Activity className="w-4 h-4" />}
                    label="Usage"
                    value={`${gpu.utilization}%`}
                  />
                  <GpuMetric
                    icon={<RefreshCw className="w-4 h-4" />}
                    label="Fan"
                    value={`${gpu.fanSpeed}%`}
                  />
                </div>
                <div className="mt-2">
                  <div className="flex justify-between text-xs text-gray-400 mb-1">
                    <span>VRAM</span>
                    <span>{gpu.memory.used}/{gpu.memory.total} MB</span>
                  </div>
                  <div className="w-full bg-gray-600 rounded h-1.5">
                    <div
                      className="bg-blue-500 rounded h-1.5"
                      style={{ width: `${(gpu.memory.used / gpu.memory.total) * 100}%` }}
                    />
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Overclock Profiles */}
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Settings className="w-5 h-5 text-purple-500" />
          Overclock Profiles
        </h2>
        <div className="grid grid-cols-3 gap-3">
          {defaultProfiles.map((profile) => (
            <button
              key={profile.name}
              onClick={() => applyProfile(profile)}
              className={`p-3 rounded-lg border transition ${
                selectedProfile === profile.name
                  ? 'border-orange-500 bg-orange-500/20'
                  : 'border-gray-600 bg-gray-700 hover:border-gray-500'
              }`}
            >
              <div className="font-medium">{profile.name}</div>
              <div className="text-xs text-gray-400 mt-1">
                Core: {profile.coreClock > 0 ? '+' : ''}{profile.coreClock}MHz
              </div>
              <div className="text-xs text-gray-400">
                Power: {profile.powerLimit}%
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Mining Schedule */}
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Clock className="w-5 h-5 text-green-500" />
          Mining Schedule
        </h2>
        <div className="flex items-center gap-4">
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={schedule.enabled}
              onChange={(e) => setSchedule({ ...schedule, enabled: e.target.checked })}
              className="w-4 h-4"
            />
            <span>Enable off-peak mining</span>
          </label>
          {schedule.enabled && (
            <>
              <div className="flex items-center gap-2">
                <span className="text-gray-400">From</span>
                <input
                  type="number"
                  value={schedule.startHour}
                  onChange={(e) => setSchedule({ ...schedule, startHour: parseInt(e.target.value) || 0 })}
                  className="w-16 px-2 py-1 bg-gray-700 rounded"
                  min="0"
                  max="23"
                />
                <span className="text-gray-400">to</span>
                <input
                  type="number"
                  value={schedule.endHour}
                  onChange={(e) => setSchedule({ ...schedule, endHour: parseInt(e.target.value) || 0 })}
                  className="w-16 px-2 py-1 bg-gray-700 rounded"
                  min="0"
                  max="23"
                />
              </div>
            </>
          )}
        </div>
        {schedule.enabled && (
          <p className="text-sm text-gray-400 mt-2">
            Mining will run from {schedule.startHour}:00 to {schedule.endHour}:00 (off-peak hours)
          </p>
        )}
      </div>
    </div>
  );
}

function StatCard({ icon, label, value, subtext }: { 
  icon: React.ReactNode; 
  label: string; 
  value: string;
  subtext?: string;
}) {
  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <div className="flex items-center gap-2 text-gray-400 mb-2">
        {icon}
        <span className="text-sm">{label}</span>
      </div>
      <div className="text-2xl font-bold">{value}</div>
      {subtext && <div className="text-xs text-gray-500">{subtext}</div>}
    </div>
  );
}

function GpuMetric({ icon, label, value, warning }: {
  icon: React.ReactNode;
  label: string;
  value: string;
  warning?: boolean;
}) {
  return (
    <div className={`flex items-center gap-1 ${warning ? 'text-red-400' : 'text-gray-300'}`}>
      {icon}
      <span className="text-gray-500">{label}:</span>
      <span className="font-medium">{value}</span>
    </div>
  );
}
