# PYRAX Analytic Dashboard

A modern, real-time network dashboard for the PYRAX Blockchain, built with Next.js 14.

## 🚀 Getting Started

### 1. Prerequisites
- Node.js 18+ installed

### 2. Installation
```bash
npm install
```

### 3. Configuration
Create a `.env.local` file in the root directory:

```env
# Point this to your VPS API address
VPS_API_URL=http://<YOUR_VPS_IP>:8080
```

### 4. Run Locally
```bash
npm run dev
```
Open [http://localhost:3000](http://localhost:3000).

## 📦 Deployment to Vercel

1. Push this code to GitHub.
2. Import the project in Vercel.
3. Add the `VPS_API_URL` environment variable in Vercel Project Settings.
4. Deploy!

## 🛠️ Features
- **Real-time Stats**: Live block height, latency, and node counts.
- **Node Explorer**: Detailed table of all discovered nodes.
- **Dark Mode**: Professional crypto aesthetic.
- **Secure Proxy**: Built-in API route to securely fetch data from HTTP VPS without Mixed Content errors.
