#!/bin/bash

# Set variables
export CHAIN_ID="uni-6"
export DENOM="ujunox"
export BECH32_HRP="juno"
export CONTRACT_CODE_PATH="artifacts/vcg_auction-aarch64.wasm"
export CONTRACT_NAME="VCG auction"
export CONTRACT_INIT_MSG='{}'
export ADMIN_ADDRESS="juno1873my89qs478e56austefw0ewpp774xmq5m4xv"
export NODE="https://rpc.uni.junonetwork.io:443"
export KEY_NAME="bluenote"
export TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block"

# Store contract code
# RES=$(junod tx wasm store $CONTRACT_CODE_PATH --from $KEY_NAME $TXFLAG --node $NODE --output json -b block)

# CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[1].value')
# echo "CODE_ID: " $CODE_ID 

CODE_ID=819

# Instantiate contract
INIT_MSG='{}'
junod tx wasm instantiate $CODE_ID "$INIT_MSG" --from $ADMIN_ADDRESS $TXFLAG --node $NODE --label $CONTRACT_NAME

# # Query contract address
# CONTRACT_ADDRESS=$(junod query wasm list-contract-by-code $CODE_ID --output json --node $JUNOD_URI | jq -r '.[0] | select(.creator == "'$ADMIN_ADDRESS'") | .address')

# echo "Contract deployed at address: $CONTRACT_ADDRESS"
