# PYRAX Deployment Infrastructure

Multi-environment deployment setup for PYRAX blockchain across 3 Digital Ocean droplets.

## Environments

| Environment | Domain | Purpose |
|------------|--------|---------|
| **Devnet** | `dev.pyrax.org` | Active development |
| **Testnet** | `testnet.pyrax.org` | End user beta testing |
| **Mainnet** | `pyrax.org` | Production |

## Services Per Environment

| Service | Devnet | Testnet | Mainnet |
|---------|--------|---------|---------|
| Explorer | `explorer.dev.pyrax.org` | `explorer.testnet.pyrax.org` | `explorer.pyrax.org` |
| RPC | `rpc.dev.pyrax.org` | `rpc.testnet.pyrax.org` | `rpc.pyrax.org` |
| API | `api.dev.pyrax.org` | `api.testnet.pyrax.org` | `api.pyrax.org` |
| Faucet | `faucet.dev.pyrax.org` | `faucet.testnet.pyrax.org` | N/A |

## SSL Configuration

Using **Cloudflare Free SSL** with Full (Strict) mode:
- Cloudflare handles SSL termination at their edge
- Origin server uses Cloudflare Origin Certificates
- All traffic encrypted end-to-end

## Quick Start

### 1. Cloudflare DNS Setup

Add these DNS records in Cloudflare (orange cloud = proxied):

```
# Devnet Droplet (IP: YOUR_DEVNET_IP)
A    dev.pyrax.org              YOUR_DEVNET_IP    Proxied
A    explorer.dev.pyrax.org     YOUR_DEVNET_IP    Proxied
A    rpc.dev.pyrax.org          YOUR_DEVNET_IP    Proxied
A    api.dev.pyrax.org          YOUR_DEVNET_IP    Proxied
A    faucet.dev.pyrax.org       YOUR_DEVNET_IP    Proxied

# Testnet Droplet (IP: YOUR_TESTNET_IP)
A    testnet.pyrax.org          YOUR_TESTNET_IP   Proxied
A    explorer.testnet.pyrax.org YOUR_TESTNET_IP   Proxied
A    rpc.testnet.pyrax.org      YOUR_TESTNET_IP   Proxied
A    api.testnet.pyrax.org      YOUR_TESTNET_IP   Proxied
A    faucet.testnet.pyrax.org   YOUR_TESTNET_IP   Proxied

# Mainnet Droplet (IP: YOUR_MAINNET_IP)
A    pyrax.org                  YOUR_MAINNET_IP   Proxied
A    www.pyrax.org              YOUR_MAINNET_IP   Proxied
A    explorer.pyrax.org         YOUR_MAINNET_IP   Proxied
A    rpc.pyrax.org              YOUR_MAINNET_IP   Proxied
A    api.pyrax.org              YOUR_MAINNET_IP   Proxied
```

### 2. Cloudflare SSL Settings

In Cloudflare Dashboard → SSL/TLS:
- Mode: **Full (Strict)**
- Always Use HTTPS: **On**
- Automatic HTTPS Rewrites: **On**
- Minimum TLS Version: **1.2**

Generate Origin Certificates:
1. Go to SSL/TLS → Origin Server
2. Create Certificate for `*.pyrax.org, pyrax.org`
3. Save the certificate and key to each droplet

### 3. Droplet Setup

SSH into each droplet and run:

```bash
# Clone the repo
git clone https://github.com/pyrax-official/pyrax.git /opt/pyrax
cd /opt/pyrax

# Run setup script (replace ENV with devnet/testnet/mainnet)
sudo ./deployment/scripts/setup.sh devnet
```

### 4. Deploy Updates

```bash
# From your local machine
./deployment/scripts/deploy.sh devnet
```

## Directory Structure

```
deployment/
├── README.md
├── env/
│   ├── devnet.env
│   ├── testnet.env
│   └── mainnet.env
├── nginx/
│   ├── devnet.conf
│   ├── testnet.conf
│   └── mainnet.conf
├── systemd/
│   ├── pyrax-node.service
│   ├── pyrax-explorer.service
│   └── pyrax-api.service
└── scripts/
    ├── setup.sh
    └── deploy.sh
```

## Ports (Internal)

| Service | Port |
|---------|------|
| pyrax-node RPC | 8545 |
| pyrax-node P2P | 30303 |
| pyrax-explorer | 3000 |
| pyrax-api | 8080 |

All external access goes through Nginx on ports 80/443.
