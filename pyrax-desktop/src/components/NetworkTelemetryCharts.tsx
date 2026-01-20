import { useEffect, useState, useRef, useCallback } from 'react';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from 'chart.js';
import { Line, Doughnut, Bar } from 'react-chartjs-2';
import { Activity, Wifi, Clock, TrendingUp, Globe, Zap } from 'lucide-react';
import { useNodeStore } from '../stores/nodeStore';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

interface TelemetryDataPoint {
  timestamp: number;
  peerCount: number;
  meshPeers: number;
  gossipPeers: number;
  inboundPeers: number;
  outboundPeers: number;
  latency: number;
  dialSuccesses: number;
  dialFailures: number;
}

const MAX_DATA_POINTS = 30; // 5 minutes of data at 10s intervals

export default function NetworkTelemetryCharts() {
  const { status, peers } = useNodeStore();
  const [telemetryHistory, setTelemetryHistory] = useState<TelemetryDataPoint[]>([]);
  const lastUpdateRef = useRef<number>(0);

  // Collect telemetry data every 10 seconds
  useEffect(() => {
    if (!status?.connected) return;

    const now = Date.now();
    if (now - lastUpdateRef.current < 10000) return; // Throttle to 10s
    lastUpdateRef.current = now;

    const newDataPoint: TelemetryDataPoint = {
      timestamp: now,
      peerCount: status.peerCount || 0,
      meshPeers: status.meshPeers || 0,
      gossipPeers: status.gossipPeers || 0,
      inboundPeers: status.inboundPeers || 0,
      outboundPeers: status.outboundPeers || 0,
      latency: status.averageRttMs || 0,
      dialSuccesses: status.dialSuccesses || 0,
      dialFailures: status.dialFailures || 0,
    };

    setTelemetryHistory(prev => {
      const newHistory = [...prev, newDataPoint];
      if (newHistory.length > MAX_DATA_POINTS) {
        return newHistory.slice(-MAX_DATA_POINTS);
      }
      return newHistory;
    });
  }, [status]);

  // Update every 10 seconds
  useEffect(() => {
    const interval = setInterval(() => {
      if (status?.connected) {
        const now = Date.now();
        lastUpdateRef.current = now;

        const newDataPoint: TelemetryDataPoint = {
          timestamp: now,
          peerCount: status.peerCount || 0,
          meshPeers: status.meshPeers || 0,
          gossipPeers: status.gossipPeers || 0,
          inboundPeers: status.inboundPeers || 0,
          outboundPeers: status.outboundPeers || 0,
          latency: status.averageRttMs || 0,
          dialSuccesses: status.dialSuccesses || 0,
          dialFailures: status.dialFailures || 0,
        };

        setTelemetryHistory(prev => {
          const newHistory = [...prev, newDataPoint];
          if (newHistory.length > MAX_DATA_POINTS) {
            return newHistory.slice(-MAX_DATA_POINTS);
          }
          return newHistory;
        });
      }
    }, 10000);

    return () => clearInterval(interval);
  }, [status]);

  if (!status?.connected) {
    return (
      <div className="bg-dark-800 rounded-xl p-6">
        <div className="text-center py-8 text-stone-400">
          <Activity size={48} className="mx-auto mb-4 opacity-50" />
          <p>Connect to a node to view network telemetry</p>
        </div>
      </div>
    );
  }

  const timeLabels = telemetryHistory.map(d => {
    const date = new Date(d.timestamp);
    return `${date.getMinutes()}:${date.getSeconds().toString().padStart(2, '0')}`;
  });

  // Peer Connection History Chart
  const peerChartData = {
    labels: timeLabels,
    datasets: [
      {
        label: 'Total Peers',
        data: telemetryHistory.map(d => d.peerCount),
        borderColor: 'rgb(139, 92, 246)',
        backgroundColor: 'rgba(139, 92, 246, 0.1)',
        fill: true,
        tension: 0.4,
      },
      {
        label: 'Mesh Peers',
        data: telemetryHistory.map(d => d.meshPeers),
        borderColor: 'rgb(34, 197, 94)',
        backgroundColor: 'rgba(34, 197, 94, 0.1)',
        fill: true,
        tension: 0.4,
      },
    ],
  };

  // Latency History Chart
  const latencyChartData = {
    labels: timeLabels,
    datasets: [
      {
        label: 'Average RTT (ms)',
        data: telemetryHistory.map(d => d.latency),
        borderColor: 'rgb(59, 130, 246)',
        backgroundColor: 'rgba(59, 130, 246, 0.1)',
        fill: true,
        tension: 0.4,
      },
    ],
  };

  // Connection Direction Doughnut
  const directionChartData = {
    labels: ['Inbound', 'Outbound'],
    datasets: [
      {
        data: [status.inboundPeers || 0, status.outboundPeers || 0],
        backgroundColor: ['rgba(59, 130, 246, 0.8)', 'rgba(139, 92, 246, 0.8)'],
        borderColor: ['rgb(59, 130, 246)', 'rgb(139, 92, 246)'],
        borderWidth: 2,
      },
    ],
  };

  // Dial Success Rate Bar Chart
  const totalDials = (status.dialSuccesses || 0) + (status.dialFailures || 0);
  const dialChartData = {
    labels: ['Successes', 'Failures'],
    datasets: [
      {
        data: [status.dialSuccesses || 0, status.dialFailures || 0],
        backgroundColor: ['rgba(34, 197, 94, 0.8)', 'rgba(239, 68, 68, 0.8)'],
        borderColor: ['rgb(34, 197, 94)', 'rgb(239, 68, 68)'],
        borderWidth: 2,
      },
    ],
  };

  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'top' as const,
        labels: {
          color: 'rgb(168, 162, 158)',
          font: { size: 11 },
        },
      },
    },
    scales: {
      x: {
        ticks: { color: 'rgb(120, 113, 108)', font: { size: 10 } },
        grid: { color: 'rgba(120, 113, 108, 0.1)' },
      },
      y: {
        ticks: { color: 'rgb(120, 113, 108)', font: { size: 10 } },
        grid: { color: 'rgba(120, 113, 108, 0.1)' },
        beginAtZero: true,
      },
    },
  };

  const doughnutOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'bottom' as const,
        labels: {
          color: 'rgb(168, 162, 158)',
          font: { size: 11 },
        },
      },
    },
  };

  // Calculate health score
  const meshHealth = status.meshPeers ? Math.min(100, (status.meshPeers / 8) * 100) : 0;
  const dialSuccess = totalDials > 0 ? ((status.dialSuccesses || 0) / totalDials) * 100 : 0;
  const latencyScore = status.averageRttMs ? Math.max(0, 100 - (status.averageRttMs / 5)) : 100;
  const overallHealth = Math.round((meshHealth + dialSuccess + latencyScore) / 3);

  return (
    <div className="space-y-6">
      {/* Network Health Overview */}
      <div className="bg-dark-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <TrendingUp size={20} className="text-pyrax-400" />
          Network Health Overview
        </h2>
        
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <HealthCard
            icon={<Activity size={20} />}
            label="Overall Health"
            value={`${overallHealth}%`}
            color={overallHealth > 70 ? 'green' : overallHealth > 40 ? 'yellow' : 'red'}
          />
          <HealthCard
            icon={<Wifi size={20} />}
            label="Mesh Health"
            value={`${Math.round(meshHealth)}%`}
            color={meshHealth > 70 ? 'green' : meshHealth > 40 ? 'yellow' : 'red'}
          />
          <HealthCard
            icon={<Globe size={20} />}
            label="Dial Success"
            value={`${Math.round(dialSuccess)}%`}
            color={dialSuccess > 70 ? 'green' : dialSuccess > 40 ? 'yellow' : 'red'}
          />
          <HealthCard
            icon={<Clock size={20} />}
            label="Latency Score"
            value={`${Math.round(latencyScore)}%`}
            color={latencyScore > 70 ? 'green' : latencyScore > 40 ? 'yellow' : 'red'}
          />
        </div>

        {/* Live Stats */}
        <div className="mt-4 pt-4 border-t border-dark-600">
          <div className="grid grid-cols-3 md:grid-cols-6 gap-3">
            <LiveStat label="NAT Status" value={status.natStatus?.split(' ')[0] || 'Unknown'} />
            <LiveStat label="Network State" value={status.networkState || 'Unknown'} />
            <LiveStat label="TCP Peers" value={`${status.peerCount || 0}/50`} />
            <LiveStat label="Mesh Peers" value={`${status.meshPeers || 0}`} />
            <LiveStat label="Gossip Peers" value={`${status.gossipPeers || 0}`} />
            <LiveStat label="Avg RTT" value={status.averageRttMs ? `${status.averageRttMs}ms` : 'N/A'} />
          </div>
        </div>
      </div>

      {/* Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Peer Connection History */}
        <div className="bg-dark-800 rounded-xl p-6">
          <h3 className="text-sm font-semibold mb-4 flex items-center gap-2 text-stone-300">
            <Wifi size={16} className="text-pyrax-400" />
            Peer Connections (Last 5 min)
          </h3>
          <div className="h-48">
            <Line data={peerChartData} options={chartOptions} />
          </div>
        </div>

        {/* Latency History */}
        <div className="bg-dark-800 rounded-xl p-6">
          <h3 className="text-sm font-semibold mb-4 flex items-center gap-2 text-stone-300">
            <Clock size={16} className="text-blue-400" />
            Network Latency (Last 5 min)
          </h3>
          <div className="h-48">
            <Line data={latencyChartData} options={chartOptions} />
          </div>
        </div>

        {/* Connection Direction */}
        <div className="bg-dark-800 rounded-xl p-6">
          <h3 className="text-sm font-semibold mb-4 flex items-center gap-2 text-stone-300">
            <Globe size={16} className="text-green-400" />
            Connection Direction
          </h3>
          <div className="h-48">
            <Doughnut data={directionChartData} options={doughnutOptions} />
          </div>
        </div>

        {/* Dial Success Rate */}
        <div className="bg-dark-800 rounded-xl p-6">
          <h3 className="text-sm font-semibold mb-4 flex items-center gap-2 text-stone-300">
            <Zap size={16} className="text-yellow-400" />
            Dial Attempts ({totalDials} total)
          </h3>
          <div className="h-48">
            <Bar data={dialChartData} options={{
              ...chartOptions,
              indexAxis: 'y' as const,
            }} />
          </div>
        </div>
      </div>

      {/* Connected Peers List */}
      {peers.length > 0 && (
        <div className="bg-dark-800 rounded-xl p-6">
          <h3 className="text-sm font-semibold mb-4 flex items-center gap-2 text-stone-300">
            <Activity size={16} className="text-green-400" />
            Active Peer Connections ({peers.length})
          </h3>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-stone-500 border-b border-dark-600">
                  <th className="pb-2">Peer ID</th>
                  <th className="pb-2">Address</th>
                  <th className="pb-2">Direction</th>
                  <th className="pb-2">Block Height</th>
                  <th className="pb-2 text-right">Latency</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-dark-700">
                {peers.slice(0, 10).map((peer) => (
                  <tr key={peer.id} className="hover:bg-dark-700/50">
                    <td className="py-2 font-mono text-xs text-stone-400">
                      {peer.id.slice(0, 20)}...
                    </td>
                    <td className="py-2 text-stone-400">{peer.address}</td>
                    <td className="py-2">
                      <span className={`text-xs px-2 py-0.5 rounded ${
                        peer.direction === 'Inbound' 
                          ? 'bg-blue-500/20 text-blue-400' 
                          : 'bg-purple-500/20 text-purple-400'
                      }`}>
                        {peer.direction}
                      </span>
                    </td>
                    <td className="py-2">{peer.bestHeight.toLocaleString()}</td>
                    <td className="py-2 text-right">
                      <span className={`font-mono ${
                        peer.latencyMs < 100 ? 'text-green-400' :
                        peer.latencyMs < 200 ? 'text-yellow-400' : 'text-red-400'
                      }`}>
                        {peer.latencyMs}ms
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {peers.length > 10 && (
              <p className="text-xs text-stone-500 mt-2 text-center">
                Showing 10 of {peers.length} peers
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function HealthCard({ icon, label, value, color }: {
  icon: React.ReactNode;
  label: string;
  value: string;
  color: 'green' | 'yellow' | 'red';
}) {
  const colorClasses = {
    green: 'text-green-400 bg-green-500/10 border-green-500/30',
    yellow: 'text-yellow-400 bg-yellow-500/10 border-yellow-500/30',
    red: 'text-red-400 bg-red-500/10 border-red-500/30',
  };

  return (
    <div className={`rounded-lg p-4 border ${colorClasses[color]}`}>
      <div className="flex items-center gap-2 mb-2">
        <span className={colorClasses[color].split(' ')[0]}>{icon}</span>
        <span className="text-xs text-stone-400">{label}</span>
      </div>
      <div className={`text-2xl font-bold ${colorClasses[color].split(' ')[0]}`}>
        {value}
      </div>
    </div>
  );
}

function LiveStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="text-center">
      <div className="text-xs text-stone-500">{label}</div>
      <div className="text-sm font-medium text-stone-200">{value}</div>
    </div>
  );
}
