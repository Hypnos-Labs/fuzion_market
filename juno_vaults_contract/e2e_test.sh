# juno github -> `sh scripts/test_node.sh c`
export KEY="juno1" 
export KEY_ADDR="juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"
export KEYALGO="secp256k1"
export JUNOD_CHAIN_ID="joe-1"
export JUNOD_KEYRING_BACKEND="test"
export JUNOD_NODE="http://localhost:26657"
export JUNOD_COMMAND_ARGS="--gas 5000000 --gas-prices="0ujuno" -y --from $KEY --broadcast-mode block --output json --chain-id juno-t1 --fees 125000ujuno --node $JUNOD_NODE"


VAULT_UPLOAD=$(junod tx wasm store artifacts/juno_vaults.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $VAULT_UPLOAD
VAULT_BASE_CODE_ID=1

# #[cw_serde]
# pub struct InstantiateMsg {
#     pub admin: Option<String>,
#     pub native_whitelist: Vec<(String, String)>,
#     pub cw20_whitelist: Vec<(String, String)>,
#     pub nft_whitelist: Vec<(String, String)>,
# }
ADMIN="$KEY_ADDR"
JSON_MSG=$(printf '{"admin":"%s","native_whitelist":[["JUNO","ujuno"]],"cw20_whitelist":[],"nft_whitelist":[]}' "$ADMIN")

VAULT_TX=$(junod tx wasm instantiate "$VAULT_BASE_CODE_ID" $JSON_MSG --label "vault" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $VAULT_TX
VAULT_ADDR=$(junod query tx $VAULT_TX --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Vault Addr: $VAULT_ADDR"


# QUERIES
# This needs to return non binary data, decode at query level
junod query wasm contract-state smart $VAULT_ADDR '{"get_admin": {}}'
junod query wasm contract-state smart $VAULT_ADDR '{"get_config": {}}'
junod query wasm contract-state smart $VAULT_ADDR '{"get_all_listings": {}}'
junod query wasm contract-state smart $VAULT_ADDR '{"get_listing_info": {"listing_id":"1"}}'
junod query wasm contract-state smart $VAULT_ADDR '{"get_listings_by_owner": {"owner":"juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"}}'
junod query wasm contract-state smart $VAULT_ADDR '{"get_buckets": {"bucket_owner":"juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"}}'
junod query wasm contract-state smart $VAULT_ADDR '{"get_listings_for_market": {"page_num":1}}'