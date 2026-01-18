# 🔥 PYRAX Inferno v0.2.47

This release includes updated Desktop Apps and CLI Tools aligned with the latest bootnode updates.

---

## 🖥️ Desktop Apps

Cross-platform desktop application for running PYRAX nodes with a graphical interface.

### Downloads

| Platform | Architecture | Download |
|----------|--------------|----------|
| **Windows** | x64 | `Inferno-Node_0.2.47_x64-setup.exe` |
| **macOS** | Intel (x64) | `Inferno-Node_0.2.47_x64.dmg` |
| **macOS** | Apple Silicon (M1/M2) | `Inferno-Node_0.2.47_aarch64.dmg` |
| **Linux** | x64 | `Inferno-Node_0.2.47_amd64.AppImage` |
| **Linux** | x64 (deb) | `Inferno-Node_0.2.47_amd64.deb` |

### Installation

1. Download the installer for your platform
2. Run the installer
3. Launch **Inferno Node** from your applications
4. The app will automatically connect to the PYRAX devnet

---

## 🔧 CLI Apps

Terminal-based node management for servers and advanced users.

### Quick Install

**Linux / macOS / WSL2:**
```bash
curl -fsSL https://get.pyrax-devnet.org/cli | bash
```

**Direct from GitHub:**
```bash
curl -fsSL https://raw.githubusercontent.com/PYRAX-Chain/PYRAX-OFFICIAL/devnet/inferno-cli/install.sh | bash
```

### CLI Features

- 🚀 Multi-instance node management
- 🔄 Automatic port conflict resolution
- 🐳 Docker container support
- 🌐 SSH remote node management
- 📊 Web-based monitoring dashboard
- 📋 Live log streaming

### Getting Started

```bash
inferno init      # Initialize node configuration
inferno start     # Start node
inferno status    # Check node status
inferno dashboard # Open web dashboard
```

See [INSTALL-CLI.md](https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/blob/devnet/docs/INSTALL-CLI.md) for detailed instructions.

---

## 📝 Changes in v0.2.47

- Fixed GossipSub mesh parameters (mesh_n_low=2) preventing node crashes
- P2P mesh now properly forms between bootnodes
- Updated CLI install scripts with pyrax-devnet.org domain
- Improved node stability and connection handling

---

## 🌐 Network Information

**Devnet Bootnodes:**
- Primary: `209.38.137.105:28545` (RPC) / `:30303` (P2P)
- Secondary: `137.184.118.228:28545` (RPC) / `:30303` (P2P)

**Chain ID:** 797292
