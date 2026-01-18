# 🍎 Inferno Node - macOS Installation Guide

## ⚠️ Important: First-Time Setup Required

Since this app is distributed outside the Mac App Store, macOS Gatekeeper will show a security warning on first launch. **This is normal** - follow the steps below to install safely.

---

## 📥 Step 1: Download

Download the `.dmg` file for your Mac:
- **Apple Silicon (M1/M2/M3)**: `Inferno-Node_x.x.x_aarch64.dmg`
- **Intel Mac**: `Inferno-Node_x.x.x_x64.dmg`

---

## 📦 Step 2: Install

1. **Double-click** the downloaded `.dmg` file
2. **Drag** "Inferno Node" to your **Applications** folder
3. **Eject** the DMG (right-click → Eject)

---

## 🚀 Step 3: First Launch (Choose ONE Method)

### Method A: Right-Click to Open ✅ (Recommended)

1. Open **Finder** → **Applications**
2. **Right-click** (or Control+click) on "Inferno Node"
3. Click **"Open"** from the menu
4. Click **"Open"** again in the security dialog

✅ After doing this once, the app will open normally from now on!

### Method B: Terminal Command

If Method A doesn't work, open **Terminal** and run:

```bash
xattr -cr /Applications/Inferno\ Node.app
```

Then double-click the app to open normally.

---

### Method C: System Settings

1. Try to open the app (it will be blocked)
2. Open **System Settings** → **Privacy & Security**
3. Scroll down and click **"Open Anyway"** next to the Inferno Node message
4. Click **"Open"** in the confirmation dialog

---

## ❓ Why is this needed?

Apple requires apps to be signed with an Apple Developer certificate ($99/year) to avoid these warnings. We're working on getting proper code signing. In the meantime, the app is **safe to use** - you can verify the source code on [GitHub](https://github.com/PYRAX-Chain/PYRAX-OFFICIAL).

---

## 🔧 Troubleshooting

### "App is damaged and can't be opened"

This is a false positive. Run this in Terminal:
```bash
xattr -cr /Applications/Inferno\ Node.app
```

### App crashes on launch

1. Make sure you're running **macOS 10.15 (Catalina)** or later
2. Try downloading the app again (file may have been corrupted)
3. Check that you have at least **500MB** free disk space

### "Inferno Node" can't be opened because it is from an unidentified developer

Use Method A (right-click → Open) or Method C (System Settings) above.

---

## 💬 Need Help?

- **Discord**: [discord.gg/pyrax](https://discord.gg/pyrax)
- **GitHub Issues**: [Report a bug](https://github.com/PYRAX-Chain/PYRAX-OFFICIAL/issues)
- **Twitter/X**: [@PYRAXChain](https://twitter.com/PYRAXChain)

---

## ✅ System Requirements

| Requirement | Minimum |
|-------------|---------|
| macOS Version | 10.15 (Catalina) or later |
| Architecture | Intel x64 or Apple Silicon (M1/M2/M3) |
| Disk Space | 500 MB |
| RAM | 4 GB |
