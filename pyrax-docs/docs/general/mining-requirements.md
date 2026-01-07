# Mining Requirements

Before starting to mine PYRAX, ensure your system meets these requirements.

## Hardware Requirements

### GPU (Graphics Card)

Your GPU is the most important component for mining.

**Minimum Requirements:**
| Specification | Minimum |
|--------------|---------|
| VRAM | 6 GB |
| GPU Type | NVIDIA GTX 1060 / AMD RX 580 |
| Age | Less than 7 years old |

**Recommended:**
| Specification | Recommended |
|--------------|-------------|
| VRAM | 8+ GB |
| GPU Type | NVIDIA RTX 3060+ / AMD RX 6700+ |
| Age | Less than 4 years old |

### Supported GPU List

**NVIDIA GPUs:**
| Series | Supported Models |
|--------|------------------|
| GTX 10 Series | 1060 6GB, 1070, 1070 Ti, 1080, 1080 Ti |
| GTX 16 Series | 1660, 1660 Super, 1660 Ti |
| RTX 20 Series | 2060, 2070, 2080, 2080 Ti |
| RTX 30 Series | 3060, 3060 Ti, 3070, 3080, 3090 |
| RTX 40 Series | 4060, 4070, 4080, 4090 |

**AMD GPUs:**
| Series | Supported Models |
|--------|------------------|
| RX 500 Series | RX 580 8GB, RX 590 |
| RX 5000 Series | RX 5600 XT, RX 5700, RX 5700 XT |
| RX 6000 Series | RX 6600, RX 6700, RX 6800, RX 6900 |
| RX 7000 Series | RX 7600, RX 7800, RX 7900 |

:::warning 4GB GPUs Not Supported
GPUs with only 4GB VRAM (like GTX 1050 Ti) cannot mine PYRAX due to DAG file size requirements.
:::

### System RAM

| Requirement | Amount |
|-------------|--------|
| Minimum | 8 GB |
| Recommended | 16 GB |

More RAM helps if running other applications alongside mining.

### Storage

| Requirement | Amount |
|-------------|--------|
| Minimum | 50 GB free |
| Recommended | 100+ GB SSD |

SSD recommended for faster operation.

### Power Supply (PSU)

Your PSU must handle your GPU(s) plus system overhead.

**Calculate Required Wattage:**
```
Required PSU = (GPU TDP × 1.2) + 200W system overhead
```

**Example:**
- RTX 3080: 320W TDP
- Calculation: (320 × 1.2) + 200 = 584W
- Recommendation: 650W+ quality PSU

**PSU Quality Matters:**
- Use 80+ Bronze or better rated
- Quality brands: Corsair, EVGA, Seasonic, be quiet!
- Avoid unknown cheap PSUs (fire risk)

### Cooling

Adequate cooling prevents thermal throttling:

**Air Cooling:**
- Case with good airflow
- Multiple case fans recommended
- Clean dust filters regularly

**GPU Temperature Targets:**
| Temperature | Status |
|-------------|--------|
| < 70°C | Excellent |
| 70-80°C | Good |
| 80-85°C | Acceptable |
| > 85°C | Too hot — improve cooling |

## Software Requirements

### Operating System

| OS | Version | Notes |
|----|---------|-------|
| Windows | 10/11 | Most user-friendly |
| Ubuntu | 20.04+ | Linux option |
| HiveOS | Latest | Dedicated mining OS |

### GPU Drivers

Keep drivers updated for best performance:

**NVIDIA:**
- Download from: [nvidia.com/drivers](https://nvidia.com/drivers)
- Recommended: Latest Game Ready or Studio drivers

**AMD:**
- Download from: [amd.com/drivers](https://amd.com/drivers)
- Enable "Compute Mode" in Radeon Settings

### Mining Software

**PYRAX Desktop (Recommended):**
- Official all-in-one solution
- Download: [pyrax.org/downloads](https://pyrax.org/downloads)

**Alternative Miners:**
| Miner | GPUs | Notes |
|-------|------|-------|
| T-Rex | NVIDIA | High performance |
| TeamRedMiner | AMD | AMD optimized |
| NBMiner | Both | Universal support |

## Network Requirements

### Internet Connection

| Requirement | Specification |
|-------------|---------------|
| Minimum speed | 10 Mbps |
| Recommended | 50+ Mbps |
| Latency | < 100ms to pool |
| Stability | Critical |

**Important:** Stable connection matters more than raw speed.

### Firewall/Router

Ensure these ports are accessible:
| Port | Protocol | Purpose |
|------|----------|---------|
| 3333 | TCP | Pool stratum |
| 4444 | TCP | Backup stratum |

## Electricity Considerations

### Power Costs

Mining profitability depends heavily on electricity rates:

| Rate ($/kWh) | Impact |
|--------------|--------|
| < $0.05 | Highly profitable |
| $0.05-0.10 | Good profitability |
| $0.10-0.15 | Moderate profitability |
| > $0.15 | Calculate carefully |

### Monthly Power Estimates

| GPU | Power Draw | Monthly Cost ($0.10/kWh) |
|-----|------------|--------------------------|
| RTX 3060 | 115W | $8.28 |
| RTX 3070 | 130W | $9.36 |
| RTX 3080 | 220W | $15.84 |
| RTX 4090 | 320W | $23.04 |

### Electrical Infrastructure

- **Standard outlet**: Usually supports 1-2 GPUs
- **Dedicated circuit**: Recommended for multiple GPUs
- **Professional installation**: Required for large operations

## Environmental Factors

### Temperature

| Ambient Temp | Effect |
|--------------|--------|
| < 25°C (77°F) | Ideal |
| 25-30°C (77-86°F) | Good |
| > 30°C (86°F) | May need extra cooling |

### Ventilation

- Mining generates significant heat
- Room must have airflow
- Consider exhaust fan for enclosed spaces

### Noise

- Mining rigs are noisy (50-70 dB)
- Place away from living spaces
- Consider basement, garage, or dedicated room

## Checklist Before Starting

### Hardware
- [ ] GPU with 6GB+ VRAM
- [ ] 8GB+ system RAM
- [ ] Adequate PSU (calculate needs)
- [ ] Proper cooling solution
- [ ] 50GB+ storage space

### Software
- [ ] Supported operating system
- [ ] Latest GPU drivers installed
- [ ] Mining software downloaded

### Network
- [ ] Stable internet connection
- [ ] Required ports accessible

### Other
- [ ] Electricity rate known
- [ ] PYRAX wallet created
- [ ] Pool selected (if pool mining)

---

:::tip Ready to Set Up?
If you meet the requirements, proceed to [Mining Setup](./mining-setup) for installation instructions.
:::
