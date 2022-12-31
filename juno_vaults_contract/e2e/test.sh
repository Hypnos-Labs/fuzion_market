# https://github.com/CosmosContracts/juno -> `sh scripts/test_node.sh c`
# Then run this from root of this directory
# sh e2e/test.sh

# TODO: 
# - timeout_commit = "500ms" for test_node.sh
# - put test into their own function?

export KEY="juno1" 
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"
export KEYALGO="secp256k1"
export JUNOD_CHAIN_ID="joe-1"
export JUNOD_KEYRING_BACKEND="test"
export JUNOD_NODE="http://localhost:26657"
export JUNOD_COMMAND_ARGS="--gas 5000000 --gas-prices="0ujuno" -y --from $KEY --broadcast-mode block --output json --chain-id juno-t1 --fees 125000ujuno --node $JUNOD_NODE"

function upload_vault {
    # == UPLOAD VAULT ==
    echo "Storing Vault contract..."
    VAULT_UPLOAD=$(junod tx wasm store artifacts/juno_vaults.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $VAULT_UPLOAD
    VAULT_BASE_CODE_ID=$(junod q tx $VAULT_UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value')

    # == INSTANTIATE ==
    ADMIN="$KEY_ADDR"
    # Do this after cw721 upload for testing cw721
    JSON_MSG=$(printf '{"admin":"%s","native_whitelist":[["JUNO","ujuno"]],"cw20_whitelist":[],"nft_whitelist":[]}' "$ADMIN")
    VAULT_TX=$(junod tx wasm instantiate "$VAULT_BASE_CODE_ID" $JSON_MSG --label "vault" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $VAULT_TX

    # == GET VAULT_CONTRACT ==
    export VAULT_CONTRACT=$(junod query tx $VAULT_TX --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Vault Addr: $VAULT_CONTRACT"
}
upload_vault

function upload_cw721 {
    echo "Storing CW721 contract..."
    TX721=$(junod tx wasm store e2e/cw721_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo "$TX721"
    CW721_CODE_ID=$(junod q tx $TX721 --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $CW721_CODE_ID"
    
    echo "Instantiating CW721 contract..."
    INIT_JSON=`printf '{"name":"e2e-test","symbol":"e2e","minter":"%s"}' $KEY_ADDR`
    IMAGE_TX_UPLOAD=$(junod tx wasm instantiate "$CW721_CODE_ID" $INIT_JSON --label "e2e-nfts-label" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $IMAGE_TX_UPLOAD
    export CW721_CONTRACT=$(junod query tx $IMAGE_TX_UPLOAD --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW721_CONTRACT: $CW721_CONTRACT"        
}
upload_cw721

function mint_cw721() {
    CONTRACT_ADDR=$1
    TOKEN_ID=$2
    OWNER=$3
    TOKEN_URI=$4
    EXECUTED_MINT_JSON=`printf '{"mint":{"token_id":"%s","owner":"%s","token_uri":"%s"}}' $TOKEN_ID $OWNER $TOKEN_URI`
    TXMINT=$(junod tx wasm execute "$CONTRACT_ADDR" "$EXECUTED_MINT_JSON" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TXMINT
}

echo "Minting NFTs..."
mint_cw721 $CW721_CONTRACT 1 $KEY_ADDR "https://m.media-amazon.com/images/I/21IAeMeSa5L.jpg"
mint_cw721 $CW721_CONTRACT 2 $KEY_ADDR "https://m.media-amazon.com/images/I/31E1mBJT-7L.jpg"

# == Helper Functions ==
function query_contract {
    junod query wasm contract-state smart $1 $2 --output json
}

function ASSERT_EQUAL() {
    # For logs, put in quotes. 
    # If $1 is from JQ, ensure its -r and don't put in quotes
    if [ "$1" != "$2" ]; then
        echo "ERROR: $1 != $2"        
    fi
}
# ===


# == INITIAL TEST ==
# Ensures CW721 was properly minted
token_uri=$(query_contract $CW721_CONTRACT '{"all_nft_info":{"token_id":"1"}}' | jq -r '.data.info.token_uri')
ASSERT_EQUAL "$token_uri" "https://m.media-amazon.com/images/I/21IAeMeSa5L.jpg"

# Ensure admin is correct from insntiation
admin=$(query_contract $VAULT_CONTRACT '{"get_config":{}}' | jq -r '.data.config.admin')
ASSERT_EQUAL "$admin" $KEY_ADDR

# Check whitelist is set to what we requested (add cw721 here in the future too)
whitelist_native=$(query_contract $VAULT_CONTRACT '{"get_config":{}}' | jq -r '.data.config.whitelist_native')
ASSERT_EQUAL $whitelist_native '[["JUNO","ujuno"]]'



# == TRANSACTIONS ==
# test cw20, nft, and native (as well as tokenfactory for v12)

function wasm_cmd {
    CONTRACT=$1
    MESSAGE=$2
    FUNDS=$3
    SHOW_LOG=${4:dont_show}
    echo "Running... $CONTRACT: $MESSAGE"

    # if length of funds is 0, then don't add funds
    if [ -z "$FUNDS" ]; then
        FUNDS=""
    else
        FUNDS="--amount $FUNDS"
    fi

    tx_hash=$(junod tx wasm execute $CONTRACT $MESSAGE $FUNDS $JUNOD_COMMAND_ARGS | jq -r '.txhash')

    export CMD_LOG=$(junod query tx $tx_hash --output json | jq -r '.raw_log')    
    if [ "$SHOW_LOG" == "show_log" ]; then
        echo "raw_log: $CMD_LOG"
    fi    
}

# type_adding: 1 = Native, 2 = CW20, 3 = NFT
wasm_cmd $VAULT_CONTRACT '{"add_to_whitelist":{"type_adding":1,"to_add":["NEW","unew"]}}' ""
# add nft
wasm_cmd $VAULT_CONTRACT `printf '{"add_to_whitelist":{"type_adding":3,"to_add":["cw721","%s"]}}' $CW721_CONTRACT`

# TODO: don't allow multiple of the same add to whitelist, ensure this errors in the future
# wasm_cmd $VAULT_CONTRACT `printf '{"add_to_whitelist":{"type_adding":3,"to_add":["cw721","%s"]}}' $CW721_CONTRACT`


# == TEST == 
# Check that whitelist was updated
native=$(query_contract $VAULT_CONTRACT '{"get_config":{}}' | jq -r '.data.config.whitelist_native')
ASSERT_EQUAL $native '[["JUNO","ujuno"],["NEW","unew"]'
# test nft
nft=$(query_contract $VAULT_CONTRACT '{"get_config":{}}' | jq -r '.data.config.whitelist_nft')
ASSERT_EQUAL $nft `printf '[["cw721","%s"]]' $CW721_CONTRACT`



# LISTINGS

# TODO: make ask Optionals?
wasm_cmd $VAULT_CONTRACT '{"create_listing":{"create_msg":{"id":"vault_1","ask":{"native":[{"denom":"ujuno","amount":"10"}],"cw20":[],"nfts":[]}}}}' "1ujuno"

# test listing went up correctly
listing_1=$(query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"vault_1"}}')
ASSERT_EQUAL "$listing_1" '{"data":{"creator":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl","status":"Being Prepared","for_sale":[["ujuno","1"]],"ask":[["ujuno","10"]],"expiration":"None"}}'

# Duplicate vault id, fails
wasm_cmd $VAULT_CONTRACT '{"create_listing":{"create_msg":{"id":"vault_1","ask":{"native":[{"denom":"ujuno","amount":"10"}],"cw20":[],"nfts":[]}}}}' "1ujuno"
ASSERT_EQUAL "$CMD_LOG" 'failed to execute message; message index: 0: ID already taken: execute wasm contract failed'

# Finalize the listing for purchase after everything is added
# TODO: optional set expire at block height?
wasm_cmd $VAULT_CONTRACT '{"finalize":{"listing_id":"vault_1","seconds":1000}}' "" show_log
# try to finalize again, will fail
wasm_cmd $VAULT_CONTRACT '{"finalize":{"listing_id":"vault_1","seconds":1000}}' ""
ASSERT_EQUAL "$CMD_LOG" 'failed to execute message; message index: 0: Listing already finalized: execute wasm contract failed'

# Createe bucket so we can purchase the listing
wasm_cmd $VAULT_CONTRACT '{"create_bucket":{"bucket_id":"buyer_a"}}' "10ujuno"
# query_contract $VAULT_CONTRACT `printf '{"get_buckets":{"bucket_owner":"%s"}}' $KEY_ADDR`


# purchase listing
wasm_cmd $VAULT_CONTRACT '{"buy_listing":{"listing_id":"vault_1","bucket_id":"buyer_a"}}' "0ujuno" show_log

# TODO: after we buy a listing (status closed), remove it from the store to save data. No need to save it. Maybe move to another store for X past listing if you want.
# Then as new listings come in, remove the oldest listing from that store

# manual queries
# TODO: add get-bucket_info?
# query_contract $VAULT_CONTRACT '{"get_config":{}}'
# query_contract $VAULT_CONTRACT '{"get_all_listings":{}}'
# query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"1"}}'
# query_contract $VAULT_CONTRACT '{"get_listings_by_owner":{"owner":"juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"}}'
# query_contract $VAULT_CONTRACT '{"get_buckets":{"bucket_owner":"juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"}}'
# query_contract $VAULT_CONTRACT '{"get_listings_for_market":{"page_num":1}}'

