#!/bin/bash
# PYRAX Deployment Script
# Usage: ./deploy.sh <environment> [component]
# Example: ./deploy.sh devnet
# Example: ./deploy.sh devnet explorer

set -e

ENV=${1:-devnet}
COMPONENT=${2:-all}
VALID_ENVS="devnet testnet mainnet"

if [[ ! " $VALID_ENVS " =~ " $ENV " ]]; then
    echo "Error: Invalid environment '$ENV'"
    echo "Usage: ./deploy.sh <devnet|testnet|mainnet> [node|explorer|api|all]"
    exit 1
fi

# Configuration
case $ENV in
    devnet)
        SERVER="root@dev.pyrax.org"
        ;;
    testnet)
        SERVER="root@testnet.pyrax.org"
        ;;
    mainnet)
        SERVER="root@pyrax.org"
        ;;
esac

REMOTE_PATH="/opt/pyrax"

echo "============================================"
echo "PYRAX Deploy - $ENV ($COMPONENT)"
echo "============================================"
echo "Server: $SERVER"
echo ""

# Function to deploy node
deploy_node() {
    echo "[Node] Syncing source code..."
    rsync -avz --delete \
        --exclude 'target' \
        --exclude '.git' \
        --exclude 'node_modules' \
        ./pyrax-node/ $SERVER:$REMOTE_PATH/pyrax-node/

    echo "[Node] Building on server..."
    ssh $SERVER "cd $REMOTE_PATH/pyrax-node && source ~/.cargo/env && cargo build --release"

    echo "[Node] Restarting service..."
    ssh $SERVER "systemctl restart pyrax-node@$ENV"

    echo "[Node] Done!"
}

# Function to deploy explorer
deploy_explorer() {
    echo "[Explorer] Building locally..."
    cd pyrax-explorer
    
    # Set environment for build
    export NEXT_PUBLIC_NETWORK=$ENV
    case $ENV in
        devnet)
            export NEXT_PUBLIC_RPC_URL="https://rpc.dev.pyrax.org"
            export NEXT_PUBLIC_API_URL="https://api.dev.pyrax.org"
            ;;
        testnet)
            export NEXT_PUBLIC_RPC_URL="https://rpc.testnet.pyrax.org"
            export NEXT_PUBLIC_API_URL="https://api.testnet.pyrax.org"
            ;;
        mainnet)
            export NEXT_PUBLIC_RPC_URL="https://rpc.pyrax.org"
            export NEXT_PUBLIC_API_URL="https://api.pyrax.org"
            ;;
    esac

    npm install
    npm run build

    echo "[Explorer] Syncing build to server..."
    rsync -avz --delete \
        .next/standalone/ $SERVER:$REMOTE_PATH/pyrax-explorer/.next/standalone/
    rsync -avz --delete \
        .next/static/ $SERVER:$REMOTE_PATH/pyrax-explorer/.next/standalone/.next/static/
    rsync -avz --delete \
        public/ $SERVER:$REMOTE_PATH/pyrax-explorer/.next/standalone/public/

    cd ..

    echo "[Explorer] Restarting service..."
    ssh $SERVER "systemctl restart pyrax-explorer@$ENV"

    echo "[Explorer] Done!"
}

# Function to deploy API
deploy_api() {
    echo "[API] API is built with node, restarting..."
    ssh $SERVER "systemctl restart pyrax-api@$ENV"
    echo "[API] Done!"
}

# Function to deploy config
deploy_config() {
    echo "[Config] Syncing deployment configs..."
    rsync -avz ./deployment/ $SERVER:$REMOTE_PATH/deployment/

    echo "[Config] Updating nginx config..."
    ssh $SERVER "cp $REMOTE_PATH/deployment/nginx/$ENV.conf /etc/nginx/sites-available/pyrax.conf"
    ssh $SERVER "nginx -t && systemctl reload nginx"

    echo "[Config] Done!"
}

# Deploy based on component
case $COMPONENT in
    node)
        deploy_node
        ;;
    explorer)
        deploy_explorer
        ;;
    api)
        deploy_api
        ;;
    config)
        deploy_config
        ;;
    all)
        deploy_config
        deploy_node
        deploy_explorer
        deploy_api
        ;;
    *)
        echo "Unknown component: $COMPONENT"
        echo "Valid components: node, explorer, api, config, all"
        exit 1
        ;;
esac

echo ""
echo "============================================"
echo "Deployment Complete!"
echo "============================================"
echo ""
echo "Check status:"
echo "  ssh $SERVER 'systemctl status pyrax-node@$ENV'"
echo "  ssh $SERVER 'systemctl status pyrax-explorer@$ENV'"
echo ""
