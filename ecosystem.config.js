module.exports = {
  apps: [
    {
      name: 'pyrax-website',
      cwd: './pyrax-website',
      script: 'npx',
      args: 'next dev -p 3000',
      interpreter: 'none',
      shell: true,
      env: {
        NODE_ENV: 'development',
        PORT: 3000
      },
      autorestart: true,
      max_restarts: 3,
      restart_delay: 5000
    },
    {
      name: 'pyrax-explorer',
      cwd: './pyrax-explorer',
      script: 'npx',
      args: 'next dev -p 3001',
      interpreter: 'none',
      shell: true,
      env: {
        NODE_ENV: 'development',
        PORT: 3001
      },
      autorestart: true,
      max_restarts: 3,
      restart_delay: 5000
    },
    {
      name: 'pyrax-docs',
      cwd: './pyrax-docs',
      script: 'npx',
      args: 'docusaurus start --port 3002',
      interpreter: 'none',
      shell: true,
      env: {
        NODE_ENV: 'development',
        PORT: 3002
      },
      autorestart: true,
      max_restarts: 3,
      restart_delay: 5000
    },
    {
      name: 'pyrax-faucet',
      cwd: './pyrax-faucet',
      script: 'npx',
      args: 'next dev -p 3003',
      interpreter: 'none',
      shell: true,
      env: {
        NODE_ENV: 'development',
        PORT: 3003
      },
      autorestart: true,
      max_restarts: 3,
      restart_delay: 5000
    },
    {
      name: 'pyrax-marketing',
      cwd: './pyrax-core-marketing',
      script: 'npx',
      args: 'next dev -p 3004',
      interpreter: 'none',
      shell: true,
      env: {
        NODE_ENV: 'development',
        PORT: 3004
      },
      autorestart: true,
      max_restarts: 3,
      restart_delay: 5000
    }
  ]
};
