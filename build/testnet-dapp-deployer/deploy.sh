#!/bin/sh

export CONSENSUS_ADDRESS=$(cat /deployments/$NETWORK/rollups.json | jq -r '.contracts.Authority.address')

echo "Deploying DApp \"$DAPP_NAME\" on network \"$NETWORK\" using consensus address \"$CONSENSUS_ADDRESS\"..."

# The machine snapshot/image is mounted directly. The hash file is in the root.
TEMPLATE_HASH_FILE="/var/opt/cartesi/machine-snapshots/hash"

echo "Using template hash from: $TEMPLATE_HASH_FILE"

cartesi-rollups create \
    --rpc "$RPC_URL" \
    --mnemonic "$MNEMONIC" \
    --deploymentFile "/deployments/$NETWORK/rollups.json" \
    --templateHashFile "$TEMPLATE_HASH_FILE" \
    --outputFile "/deployments/$NETWORK/$DAPP_NAME.json" \
    --consensusAddress "$CONSENSUS_ADDRESS"
