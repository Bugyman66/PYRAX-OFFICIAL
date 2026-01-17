import { useState } from 'react';
import { 
  Network as NetworkIcon, 
  Shield, 
  Router, 
  Globe, 
  CheckCircle2, 
  XCircle, 
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  ExternalLink,
  Copy,
  Check
} from 'lucide-react';
import { useNodeStore } from '../stores/nodeStore';

interface RouterGuide {
  name: string;
  steps: string[];
  notes?: string;
}

const routerGuides: RouterGuide[] = [
  {
    name: 'Generic Router (Most Common)',
    steps: [
      'Open your browser and go to your router\'s admin page (usually 192.168.1.1 or 192.168.0.1)',
      'Log in with your router credentials (check the sticker on your router if unsure)',
      'Find "Port Forwarding", "NAT", "Virtual Server", or "Applications & Gaming" section',
      'Click "Add New" or "Create Rule"',
      'Enter: Service Name: "PYRAX Node", Protocol: TCP, External Port: 30303, Internal Port: 30303',
      'Enter your computer\'s local IP address (found via ipconfig/ifconfig)',
      'Save and apply the changes'
    ]
  },
  {
    name: 'Netgear Routers',
    steps: [
      'Go to routerlogin.net or 192.168.1.1 in your browser',
      'Login with admin credentials',
      'Navigate to Advanced → Advanced Setup → Port Forwarding/Port Triggering',
      'Select "Add Custom Service"',
      'Service Name: PYRAX, Protocol: TCP, External Port: 30303, Internal Port: 30303',
      'Enter your computer\'s IP address',
      'Click Apply'
    ]
  },
  {
    name: 'TP-Link Routers',
    steps: [
      'Go to 192.168.0.1 or tplinkwifi.net',
      'Login with your credentials',
      'Navigate to Advanced → NAT Forwarding → Virtual Servers',
      'Click "Add"',
      'Service Type: Custom, External Port: 30303, Internal Port: 30303, Protocol: TCP',
      'Enter your device IP and click Save'
    ]
  },
  {
    name: 'ASUS Routers',
    steps: [
      'Go to router.asus.com or 192.168.1.1',
      'Login with your credentials',
      'Navigate to WAN → Virtual Server / Port Forwarding',
      'Enable Port Forwarding',
      'Add: Service Name: PYRAX, Port Range: 30303, Local IP: [your IP], Local Port: 30303, Protocol: TCP',
      'Click Apply'
    ]
  },
  {
    name: 'Linksys Routers',
    steps: [
      'Go to 192.168.1.1 in your browser',
      'Login with admin credentials',
      'Navigate to Apps and Gaming → Single Port Forwarding',
      'Application Name: PYRAX, External Port: 30303, Internal Port: 30303, Protocol: TCP',
      'Enter Device IP and check Enabled',
      'Click Save Settings'
    ]
  },
  {
    name: 'Xfinity/Comcast Gateway',
    steps: [
      'Go to 10.0.0.1 in your browser',
      'Login with admin/password (or check gateway sticker)',
      'Navigate to Advanced → Port Forwarding',
      'Click "Add Service"',
      'Common Service: Other, Service Name: PYRAX',
      'Server IP: [your computer IP], Start/End Port: 30303, Protocol: TCP',
      'Click Save'
    ],
    notes: 'You may need to disable "Advanced Security" temporarily for port forwarding to work.'
  },
  {
    name: 'AT&T U-verse Gateway',
    steps: [
      'Go to 192.168.1.254 in your browser',
      'Login with your gateway credentials',
      'Navigate to Settings → Firewall → Applications, Pinholes and DMZ',
      'Select your device and click "Add Application"',
      'Create custom application: Name: PYRAX, Protocol: TCP, Port: 30303-30303',
      'Add to your device and save'
    ]
  },
  {
    name: 'Google Wifi / Nest Wifi',
    steps: [
      'Open the Google Home app on your phone',
      'Tap on Wifi → Settings → Advanced networking → Port management',
      'Tap the + button to add a new rule',
      'Select your computer from the device list',
      'Enter: Internal port: 30303, External port: 30303, Protocol: TCP',
      'Save the rule'
    ]
  }
];

const firewallGuides = {
  windows: {
    name: 'Windows Firewall',
    steps: [
      'Press Windows + R, type "wf.msc" and press Enter',
      'Click "Inbound Rules" in the left panel',
      'Click "New Rule..." in the right panel',
      'Select "Port" and click Next',
      'Select "TCP" and enter "30303" for Specific local ports',
      'Select "Allow the connection" and click Next',
      'Check all profiles (Domain, Private, Public) and click Next',
      'Name the rule "PYRAX Node P2P" and click Finish'
    ],
    alternative: 'Or run as Administrator: netsh advfirewall firewall add rule name="PYRAX Node P2P" dir=in action=allow protocol=TCP localport=30303'
  },
  macos: {
    name: 'macOS Firewall',
    steps: [
      'Open System Preferences → Security & Privacy → Firewall',
      'Click the lock icon and enter your password',
      'Click "Firewall Options..."',
      'Click the "+" button and add Inferno Node application',
      'Set it to "Allow incoming connections"',
      'Click OK and close preferences'
    ],
    alternative: 'macOS typically allows outbound connections. If using a third-party firewall like Little Snitch, add a rule to allow TCP port 30303.'
  },
  linux: {
    name: 'Linux Firewall (UFW)',
    steps: [
      'Open terminal',
      'Run: sudo ufw allow 30303/tcp',
      'Run: sudo ufw reload',
      'Verify with: sudo ufw status'
    ],
    alternative: 'For iptables: sudo iptables -A INPUT -p tcp --dport 30303 -j ACCEPT'
  }
};

export default function Network() {
  const { status } = useNodeStore();
  const [expandedRouter, setExpandedRouter] = useState<string | null>(null);
  const [expandedFirewall, setExpandedFirewall] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const peerCount = status?.peerCount || 0;
  const isConnected = status?.connected || false;
  const meshHealthy = peerCount >= 6;

  return (
    <div className="p-6 space-y-6 max-w-4xl">
      <div className="flex items-center gap-3">
        <NetworkIcon className="w-8 h-8 text-pyrax-500" />
        <div>
          <h1 className="text-2xl font-bold text-white">Network Configuration</h1>
          <p className="text-stone-400">Configure your firewall and router for optimal P2P connectivity</p>
        </div>
      </div>

      {/* Connection Status */}
      <div className="bg-dark-800 rounded-xl p-6 border border-dark-600">
        <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <Globe className="w-5 h-5 text-pyrax-500" />
          P2P Connection Status
        </h2>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-dark-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              {isConnected ? (
                <CheckCircle2 className="w-5 h-5 text-green-500" />
              ) : (
                <XCircle className="w-5 h-5 text-red-500" />
              )}
              <span className="text-stone-300 font-medium">Bootnode Connection</span>
            </div>
            <p className={`text-sm ${isConnected ? 'text-green-400' : 'text-red-400'}`}>
              {isConnected ? 'Connected' : 'Disconnected'}
            </p>
          </div>

          <div className="bg-dark-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              {meshHealthy ? (
                <CheckCircle2 className="w-5 h-5 text-green-500" />
              ) : peerCount > 0 ? (
                <AlertTriangle className="w-5 h-5 text-yellow-500" />
              ) : (
                <XCircle className="w-5 h-5 text-red-500" />
              )}
              <span className="text-stone-300 font-medium">Peer Mesh</span>
            </div>
            <p className={`text-sm ${meshHealthy ? 'text-green-400' : peerCount > 0 ? 'text-yellow-400' : 'text-red-400'}`}>
              {peerCount} / 6 minimum peers
            </p>
          </div>

          <div className="bg-dark-700 rounded-lg p-4">
            <div className="flex items-center gap-2 mb-2">
              <Shield className="w-5 h-5 text-pyrax-500" />
              <span className="text-stone-300 font-medium">Required Port</span>
            </div>
            <p className="text-sm text-pyrax-400 font-mono">TCP 30303</p>
          </div>
        </div>

        {!meshHealthy && (
          <div className="mt-4 p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
            <div className="flex items-start gap-3">
              <AlertTriangle className="w-5 h-5 text-yellow-500 mt-0.5" />
              <div>
                <p className="text-yellow-400 font-medium">Low Peer Count</p>
                <p className="text-stone-400 text-sm mt-1">
                  Your node needs at least 6 peers for the gossip mesh to function properly. 
                  This usually means other peers cannot reach your node due to firewall or NAT restrictions.
                  Follow the guides below to open port 30303.
                </p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Why Port Forwarding */}
      <div className="bg-dark-800 rounded-xl p-6 border border-dark-600">
        <h2 className="text-lg font-semibold text-white mb-4">Why is Port Forwarding Required?</h2>
        <div className="space-y-3 text-stone-300 text-sm">
          <p>
            PYRAX uses a peer-to-peer (P2P) network for block and transaction propagation. 
            For optimal connectivity, your node needs to accept <strong className="text-white">incoming connections</strong> from other nodes.
          </p>
          <p>
            Most home networks use NAT (Network Address Translation), which blocks incoming connections by default. 
            Port forwarding tells your router to allow connections on port <code className="bg-dark-600 px-1.5 py-0.5 rounded text-pyrax-400">30303</code> through to your computer.
          </p>
          <div className="bg-dark-700 rounded-lg p-4 mt-4">
            <p className="text-stone-400">
              <strong className="text-white">Without port forwarding:</strong> Your node can connect OUT to the bootnode, but other nodes cannot connect IN to you. 
              This means you can receive blocks but won't be part of the gossip mesh.
            </p>
          </div>
        </div>
      </div>

      {/* Step 1: Firewall */}
      <div className="bg-dark-800 rounded-xl p-6 border border-dark-600">
        <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <Shield className="w-5 h-5 text-pyrax-500" />
          Step 1: Configure Your Firewall
        </h2>
        <p className="text-stone-400 text-sm mb-4">
          First, allow incoming connections on port 30303 through your computer's firewall.
        </p>

        <div className="space-y-3">
          {Object.entries(firewallGuides).map(([key, guide]) => (
            <div key={key} className="border border-dark-600 rounded-lg overflow-hidden">
              <button
                onClick={() => setExpandedFirewall(expandedFirewall === key ? null : key)}
                className="w-full flex items-center justify-between p-4 bg-dark-700 hover:bg-dark-650 transition-colors"
              >
                <span className="font-medium text-white">{guide.name}</span>
                {expandedFirewall === key ? (
                  <ChevronDown className="w-5 h-5 text-stone-400" />
                ) : (
                  <ChevronRight className="w-5 h-5 text-stone-400" />
                )}
              </button>
              {expandedFirewall === key && (
                <div className="p-4 bg-dark-750 space-y-4">
                  <ol className="list-decimal list-inside space-y-2 text-stone-300 text-sm">
                    {guide.steps.map((step, i) => (
                      <li key={i}>{step}</li>
                    ))}
                  </ol>
                  {guide.alternative && (
                    <div className="mt-4 p-3 bg-dark-600 rounded-lg">
                      <p className="text-xs text-stone-400 mb-2">Quick command:</p>
                      <div className="flex items-center gap-2">
                        <code className="flex-1 text-xs text-pyrax-400 font-mono break-all">
                          {guide.alternative}
                        </code>
                        <button
                          onClick={() => copyToClipboard(guide.alternative!)}
                          className="p-1.5 hover:bg-dark-500 rounded transition-colors"
                          title="Copy command"
                        >
                          {copied ? (
                            <Check className="w-4 h-4 text-green-500" />
                          ) : (
                            <Copy className="w-4 h-4 text-stone-400" />
                          )}
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Step 2: Router Port Forwarding */}
      <div className="bg-dark-800 rounded-xl p-6 border border-dark-600">
        <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <Router className="w-5 h-5 text-pyrax-500" />
          Step 2: Configure Router Port Forwarding
        </h2>
        <p className="text-stone-400 text-sm mb-4">
          After configuring your firewall, set up port forwarding on your router to allow external connections.
        </p>

        {/* Quick Reference */}
        <div className="bg-dark-700 rounded-lg p-4 mb-4">
          <h3 className="text-white font-medium mb-2">Port Forwarding Settings</h3>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div className="text-stone-400">Protocol:</div>
            <div className="text-white font-mono">TCP</div>
            <div className="text-stone-400">External Port:</div>
            <div className="text-white font-mono">30303</div>
            <div className="text-stone-400">Internal Port:</div>
            <div className="text-white font-mono">30303</div>
            <div className="text-stone-400">Internal IP:</div>
            <div className="text-pyrax-400 font-mono">[Your Computer's IP]</div>
          </div>
        </div>

        {/* Find Your IP */}
        <div className="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4 mb-4">
          <p className="text-blue-400 text-sm">
            <strong>Find your local IP address:</strong><br />
            Windows: Open Command Prompt, type <code className="bg-dark-600 px-1 rounded">ipconfig</code><br />
            macOS/Linux: Open Terminal, type <code className="bg-dark-600 px-1 rounded">ifconfig</code> or <code className="bg-dark-600 px-1 rounded">ip addr</code>
          </p>
        </div>

        <div className="space-y-3">
          {routerGuides.map((guide) => (
            <div key={guide.name} className="border border-dark-600 rounded-lg overflow-hidden">
              <button
                onClick={() => setExpandedRouter(expandedRouter === guide.name ? null : guide.name)}
                className="w-full flex items-center justify-between p-4 bg-dark-700 hover:bg-dark-650 transition-colors"
              >
                <span className="font-medium text-white">{guide.name}</span>
                {expandedRouter === guide.name ? (
                  <ChevronDown className="w-5 h-5 text-stone-400" />
                ) : (
                  <ChevronRight className="w-5 h-5 text-stone-400" />
                )}
              </button>
              {expandedRouter === guide.name && (
                <div className="p-4 bg-dark-750">
                  <ol className="list-decimal list-inside space-y-2 text-stone-300 text-sm">
                    {guide.steps.map((step, i) => (
                      <li key={i}>{step}</li>
                    ))}
                  </ol>
                  {guide.notes && (
                    <div className="mt-3 p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
                      <p className="text-yellow-400 text-xs">{guide.notes}</p>
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Verification */}
      <div className="bg-dark-800 rounded-xl p-6 border border-dark-600">
        <h2 className="text-lg font-semibold text-white mb-4">Step 3: Verify Your Configuration</h2>
        <div className="space-y-4 text-stone-300 text-sm">
          <p>After configuring your firewall and router:</p>
          <ol className="list-decimal list-inside space-y-2">
            <li>Restart the Inferno Node application</li>
            <li>Wait 1-2 minutes for peer discovery</li>
            <li>Check the P2P Connection Status above - you should see more than 6 peers</li>
          </ol>
          
          <div className="mt-4 p-4 bg-dark-700 rounded-lg">
            <p className="text-stone-400 mb-2">You can also verify your port is open using an online port checker:</p>
            <a 
              href="https://www.yougetsignal.com/tools/open-ports/"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 text-pyrax-400 hover:text-pyrax-300 transition-colors"
            >
              Check Port 30303 <ExternalLink className="w-4 h-4" />
            </a>
          </div>
        </div>
      </div>

      {/* Troubleshooting */}
      <div className="bg-dark-800 rounded-xl p-6 border border-dark-600">
        <h2 className="text-lg font-semibold text-white mb-4">Troubleshooting</h2>
        <div className="space-y-4 text-sm">
          <div className="p-4 bg-dark-700 rounded-lg">
            <p className="text-white font-medium mb-2">Still showing 0-1 peers?</p>
            <ul className="list-disc list-inside text-stone-400 space-y-1">
              <li>Make sure you forwarded TCP (not UDP) port 30303</li>
              <li>Double-check your computer's local IP address is correct</li>
              <li>Some ISPs use CGNAT - contact them to get a public IP or ask about port forwarding</li>
              <li>If using a VPN, disable it or configure it to allow port 30303</li>
              <li>Restart your router after making changes</li>
            </ul>
          </div>
          
          <div className="p-4 bg-dark-700 rounded-lg">
            <p className="text-white font-medium mb-2">Using Double NAT?</p>
            <p className="text-stone-400">
              If you have multiple routers (e.g., ISP modem + personal router), you need to configure port forwarding on BOTH devices.
              Forward port 30303 from your ISP modem to your personal router, then from your router to your computer.
            </p>
          </div>

          <div className="p-4 bg-dark-700 rounded-lg">
            <p className="text-white font-medium mb-2">Dynamic IP Address?</p>
            <p className="text-stone-400">
              Your computer's local IP may change. Consider setting a static IP address for your computer, 
              or use DHCP reservation in your router settings to always assign the same IP.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
