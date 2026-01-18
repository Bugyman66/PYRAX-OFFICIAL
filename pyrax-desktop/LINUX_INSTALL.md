# 🐧 Inferno Node - Linux Installation Guide

Choose your preferred installation method below.

---

## 📥 Option 1: AppImage (Recommended - Works on All Distros)

### Step 1: Download
Download `Inferno-Node_x.x.x_amd64.AppImage` from the [Releases page](https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases)

### Step 2: Make Executable
```bash
chmod +x Inferno-Node_*.AppImage
```

### Step 3: Run
```bash
./Inferno-Node_*.AppImage
```

✅ That's it! The AppImage is self-contained and works on most Linux distributions.

---

## 📦 Option 2: .deb Package (Debian/Ubuntu/Pop!_OS/Linux Mint)

### Step 1: Download
Download `Inferno-Node_x.x.x_amd64.deb` from the [Releases page](https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/releases)

### Step 2: Install
```bash
# Using dpkg
sudo dpkg -i Inferno-Node_*.deb

# Fix any missing dependencies
sudo apt-get install -f
```

### Step 3: Run
Launch from your **Applications menu** or run:
```bash
inferno-node
```

---

## 🔧 Troubleshooting

### AppImage: "FUSE not found" or "cannot mount"

Install FUSE (required for AppImages):

```bash
# Ubuntu/Debian/Pop!_OS
sudo apt-get install fuse libfuse2

# Fedora
sudo dnf install fuse

# Arch Linux
sudo pacman -S fuse2

# openSUSE
sudo zypper install fuse
```

### AppImage: "Permission denied"

Make the file executable:
```bash
chmod +x Inferno-Node_*.AppImage
```

### Missing WebKit/GTK Libraries

Install the required dependencies:

```bash
# Ubuntu/Debian (20.04+)
sudo apt-get install libwebkit2gtk-4.0-37 libgtk-3-0

# Ubuntu 24.04+ / Debian 13+
sudo apt-get install libwebkit2gtk-4.1-0 libgtk-3-0

# Fedora
sudo dnf install webkit2gtk3 gtk3

# Arch Linux
sudo pacman -S webkit2gtk gtk3

# openSUSE
sudo zypper install libwebkit2gtk-4_0-37 gtk3
```

### App Crashes on Start

Run from terminal to see error messages:
```bash
./Inferno-Node_*.AppImage 2>&1 | tee crash.log
```

Share `crash.log` when reporting issues.

### "cannot execute binary file"

You may have downloaded the wrong architecture. Make sure you download the `amd64` (x86_64) version.

---

## ✅ System Requirements

| Requirement | Minimum |
|-------------|---------|
| Distribution | Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch, or equivalent |
| Architecture | x86_64 (AMD64) |
| Dependencies | GTK 3, WebKit2GTK 4.0+ |
| Disk Space | 500 MB |
| RAM | 4 GB |

---

## 🔒 Verifying the Download (Optional)

Check the file integrity using the signature file:
```bash
# Download the .sig file alongside the AppImage/deb
# Verify with our public key (coming soon)
```

---

## 💬 Need Help?

- **Discord**: [discord.gg/pyrax](https://discord.gg/pyrax)
- **GitHub Issues**: [Report a bug](https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/issues)
- **Twitter/X**: [@PYRAXChain](https://twitter.com/PYRAXChain)

---

## 🖥️ Desktop Integration (Optional)

To add Inferno Node to your application menu with the AppImage:

```bash
# Create desktop entry
cat > ~/.local/share/applications/inferno-node.desktop << EOF
[Desktop Entry]
Name=Inferno Node
Exec=/path/to/Inferno-Node_*.AppImage
Icon=inferno-node
Type=Application
Categories=Network;Finance;
EOF

# Update desktop database
update-desktop-database ~/.local/share/applications/
```

Replace `/path/to/` with the actual path to your AppImage.
