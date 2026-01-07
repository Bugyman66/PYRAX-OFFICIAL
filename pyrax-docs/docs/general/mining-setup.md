# Mining Setup Guide

This step-by-step guide will help you set up PYRAX mining on your computer.

## Method 1: PYRAX Desktop (Recommended)

The official PYRAX Desktop application is the easiest way to start mining.

### Step 1: Download PYRAX Desktop

1. Visit [pyrax.org/downloads](https://pyrax.org/downloads)
2. Select your operating system:
   - Windows (64-bit)
   - macOS
   - Linux (Ubuntu/Debian)
3. Download the installer

### Step 2: Install the Application

**Windows:**
1. Run the downloaded `.exe` file
2. Click "Yes" if prompted by Windows Security
3. Follow the installation wizard
4. Launch PYRAX Desktop

**macOS:**
1. Open the downloaded `.dmg` file
2. Drag PYRAX Desktop to Applications
3. Open from Applications (right-click → Open if security warning)

**Linux:**
```bash
# Make executable
chmod +x pyrax-desktop.AppImage
# Run
./pyrax-desktop.AppImage
```

### Step 3: Create or Import Wallet

**Create New Wallet:**
1. Click "Create New Wallet"
2. Set a strong password
3. **Write down your 12-word seed phrase**
4. Verify by entering seed words
5. Your wallet is ready

**Import Existing Wallet:**
1. Click "Import Wallet"
2. Enter your seed phrase
3. Set a password
4. Click "Import"

### Step 4: Configure Mining

1. Click "Mining" in the sidebar
2. Enter your wallet address (auto-filled if using integrated wallet)
3. Select mining pool or solo mining
4. Adjust settings:
   - Power limit (%)
   - Enable/disable Stream B (AI compute)

### Step 5: Start Mining

1. Click "Start Mining"
2. Wait for initialization (DAG generation)
3. Monitor your hashrate and earnings
4. That's it — you're mining!

## Method 2: Third-Party Miners

For advanced users who prefer direct miner control.

### T-Rex Miner (NVIDIA)

**Download:**
1. Visit [github.com/trexminer/T-Rex](https://github.com/trexminer/T-Rex)
2. Download latest release
3. Extract to a folder

**Configuration:**
Create a batch file `start-pyrax.bat`:
```batch
t-rex -a kawpow -o stratum+tcp://pool.pyrax.org:3333 -u YOUR_WALLET_ADDRESS -p x
pause
```

**Run:**
1. Double-click `start-pyrax.bat`
2. Miner will start
3. Monitor hashrate in console

### TeamRedMiner (AMD)

**Download:**
1. Visit [github.com/todxx/teamredminer](https://github.com/todxx/teamredminer)
2. Download latest release
3. Extract to a folder

**Configuration:**
Create a batch file `start-pyrax.bat`:
```batch
teamredminer -a kawpow -o stratum+tcp://pool.pyrax.org:3333 -u YOUR_WALLET_ADDRESS -p x
pause
```

### NBMiner (NVIDIA & AMD)

**Download:**
1. Visit [github.com/NebuTech/NBMiner](https://github.com/NebuTech/NBMiner)
2. Download latest release
3. Extract to a folder

**Configuration:**
Create a batch file `start-pyrax.bat`:
```batch
nbminer -a kawpow -o stratum+tcp://pool.pyrax.org:3333 -u YOUR_WALLET_ADDRESS
pause
```

## Pool Configuration

### Official Pools

| Pool | Address | Port |
|------|---------|------|
| Primary | pool.pyrax.org | 3333 |
| Backup | pool2.pyrax.org | 3333 |

### Pool Connection String

```
stratum+tcp://pool.pyrax.org:3333
```

### Worker Names (Optional)

Add a worker name to identify rigs:
```
YOUR_WALLET_ADDRESS.worker1
YOUR_WALLET_ADDRESS.rig2
```

## Optimizing Performance

### Power Settings

Reduce power for better efficiency:

**NVIDIA (using nvidia-smi):**
```bash
# Set power limit to 75%
nvidia-smi -pl 225  # Adjust based on your GPU
```

**Using MSI Afterburner:**
1. Open MSI Afterburner
2. Set Power Limit to 70-80%
3. Apply settings

### Memory Overclock

KAWPOW benefits from faster memory:

| GPU Type | Suggested Memory OC |
|----------|-------------------|
| NVIDIA | +500 to +1200 MHz |
| AMD | Fast Timings enabled |

:::warning
Start conservative and increase gradually. Unstable overclock causes rejected shares.
:::

### Temperature Management

**Target temperatures:**
- Core: < 75°C
- Memory (if shown): < 95°C

**Tips:**
- Increase fan speed
- Improve case airflow
- Replace thermal paste (older GPUs)
- Consider aftermarket cooling

## Monitoring Your Miner

### Key Metrics

| Metric | Description | Good Value |
|--------|-------------|------------|
| Hashrate | Mining speed | Varies by GPU |
| Accepted Shares | Valid work submitted | 99%+ |
| Rejected Shares | Invalid work | < 1% |
| Stale Shares | Late submissions | < 0.5% |
| Temperature | GPU heat | < 80°C |
| Power | Energy usage | As configured |

### Monitoring Tools

**PYRAX Desktop:**
- Built-in dashboard
- Real-time statistics
- Earnings tracker

**Pool Dashboard:**
- Visit pool website
- Enter your wallet address
- View detailed statistics

**External Tools:**
- GPU-Z (hardware monitoring)
- HWiNFO (detailed sensors)
- MSI Afterburner (OC and monitoring)

## Troubleshooting

### "No CUDA devices found"

**Cause:** Driver issue or GPU not detected

**Solutions:**
1. Update NVIDIA drivers
2. Restart computer
3. Check GPU is properly seated
4. Verify GPU works in other applications

### Low Hashrate

**Possible causes:**
- Thermal throttling (too hot)
- Power limit too low
- Background applications using GPU
- Outdated drivers

**Solutions:**
1. Check temperatures
2. Increase power limit
3. Close other GPU applications
4. Update drivers

### High Rejected Shares

**Possible causes:**
- Unstable overclock
- Network issues
- Wrong pool settings

**Solutions:**
1. Reduce memory overclock
2. Check internet stability
3. Verify pool configuration

### Miner Crashes

**Possible causes:**
- Unstable settings
- Insufficient power
- Driver issues
- Memory errors

**Solutions:**
1. Reset to stock settings
2. Check PSU capacity
3. Reinstall drivers
4. Test GPU stability

### DAG Generation Slow

**Cause:** Normal on first start or epoch change

**Solution:** Wait patiently — can take several minutes

## Running 24/7

### Auto-Start Mining

**Windows:**
1. Create shortcut to mining batch file
2. Press Win+R, type `shell:startup`
3. Move shortcut to startup folder

**Linux:**
1. Create systemd service
2. Enable auto-start

### Watchdog Scripts

Monitor and restart if miner crashes:
- Use built-in miner watchdog
- Or external monitoring tools

### Remote Monitoring

- Pool dashboards (access from any device)
- PYRAX Desktop remote features
- Third-party mining monitors

## Security Notes

1. **Never share your wallet seed phrase**
2. **Download software from official sources only**
3. **Keep mining software updated**
4. **Use antivirus but add miner exceptions**
5. **Secure remote access if enabled**

---

:::tip You're Mining!
Once your miner is running and showing accepted shares, congratulations — you're earning PYRAX! Check the pool dashboard to track your earnings.
:::
