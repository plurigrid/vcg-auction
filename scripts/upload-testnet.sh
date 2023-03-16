# Set variables
export CHAIN_ID="uni-6"
export DENOM="ujunox"
export BECH32_HRP="juno"
export CONTRACT_CODE_PATH="artifacts/vcg_auction-aarch64.wasm"
export CONTRACT_NAME="vcg-auction"
export CONTRACT_INIT_MSG='{}'
export ADMIN_ADDRESS=""
export NODE="https://rpc.uni.junonetwork.io:443"
export KEY_NAME="<key name>"
export TXFLAG="--chain-id ${CHAIN_ID} --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block"
export BIDDER_ADDRESS="<bidder address>"
CONTRACT_ADDRESS=juno1xsv7xkwakhha392s4zq0h7kzdmhqhqd67aknwv3vcy3dhqpw46cq90yvtq

# Store contract code
RES=$(junod tx wasm store $CONTRACT_CODE_PATH --from $KEY_NAME $TXFLAG --node $NODE --output json -b block)

CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[1].value')
echo "CODE_ID: " $CODE_ID 

# Instantiate contract
OUTPUT=$(junod tx wasm instantiate $CODE_ID "$CONTRACT_INIT_MSG" --from $KEY_NAME $TXFLAG --node "https://rpc.uni.junonetwork.io:443" --label $CONTRACT_NAME --admin $ADMIN_ADDRESS)

CONTRACT_ADDRESS=$(echo $OUTPUT | jq -r '.logs[0].events[0].attributes[] | select(.key == "_contract_address").value')
echo "CONTRACT ADDRESS: " $CONTRACT_ADDRESS

# Start auction

junod tx wasm execute $CONTRACT_ADDRESS '{"execute_start_auction": {"name": "test auction", "max_num_participants": "10"}}' --from $KEY_NAME --node "https://rpc.uni.junonetwork.io:443" $TXFLAG

# Bid in auction

junod tx wasm execute $CONTRACT_ADDRESS '{"execute_bid": {"bid_amount": "10"}}' --from $KEY_NAME --node "https://rpc.uni.junonetwork.io:443" $TXFLAG

# Query auction info

junod query wasm contract-state smart $CONTRACT_ADDRESS '{"query_get_bids_for_bidder": {"bidder": "'$BIDDER_ADDRESS'"}}' --node "https://rpc.uni.junonetwork.io:443" 

junod query wasm contract-state smart $CONTRACT_ADDRESS '{"query_get_bids_for_auction": {"auction_id": "1"}}' --node "https://rpc.uni.junonetwork.io:443"