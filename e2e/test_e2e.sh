# used as our test account
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"

# export KEY="juno1" 
# export KEYALGO="secp256k1"
# export JUNOD_CHAIN_ID="junod-1"
# export JUNOD_KEYRING_BACKEND="test"
# export JUNOD_NODE="http://localhost:26657"
# export JUNOD_COMMAND_ARGS="--gas 5000000 --gas-prices="0ujuno" -y --from $KEY --broadcast-mode block --output json --chain-id juno-t1 --fees 125000ujuno --node $JUNOD_NODE"



IMAGE_TAG=${2:-"v9.0.0"}
CONTAINER_NAME="juno_cw_vaults"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
JUNOD_COMMAND_ARGS="--gas-prices 0.1$DENOM --gas auto --gas-adjustment 1.3 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE"
BLOCK_GAS_LIMIT=${GAS_LIMIT:-100000000} # mirrors mainnet

echo "Building $IMAGE_TAG"
echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

docker kill $CONTAINER_NAME
docker volume rm -f junod_data

# run junod docker
docker run --rm -d --name $CONTAINER_NAME \
    -e STAKE_TOKEN=$DENOM \
    -e GAS_LIMIT="$GAS_LIMIT" \
    -e UNSAFE_CORS=true \
    -p 1317:1317 -p 26656:26656 -p 26657:26657 \
    --mount type=volume,source=junod_data,target=/root \
    ghcr.io/cosmoscontracts/juno:$IMAGE_TAG /opt/setup_and_run.sh $KEY_ADDR

# compile vaults contract
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.11

# copy wasm to docker container
docker cp artifacts/juno_vaults.wasm $CONTAINER_NAME:/juno_vaults.wasm
# copy helper contracts to container
docker cp e2e/cw20_base.wasm $CONTAINER_NAME:/cw20_base.wasm
docker cp e2e/cw721_base.wasm $CONTAINER_NAME:/cw721_base.wasm

# validator addr
VALIDATOR_ADDR=$($BINARY keys show validator --address)
echo "Validator address:"
echo $VALIDATOR_ADDR

BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR)
echo "Pre-store balance:"
echo $BALANCE_1

echo "Address to deploy contracts: $KEY_ADDR"
echo "JUNOD_COMMAND_ARGS: $JUNOD_COMMAND_ARGS"

# errors from this point on are no bueno
set -e

# provision juno default user i.e. juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y, account already has funds in the docker container
echo "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose" | $BINARY keys add test-user --recover --keyring-backend test
export TEST_ADDR=$($BINARY keys show test-user --address --keyring-backend test)


function upload_vault {
    # == UPLOAD VAULT ==
    echo "Storing Vault contract..."
    VAULT_UPLOAD=$(BINARY tx wasm store /juno_vaults.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $VAULT_UPLOAD
    VAULT_BASE_CODE_ID=$(BINARY q tx $VAULT_UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value')

    # == INSTANTIATE ==
    ADMIN="$KEY_ADDR"
    # Do this after cw721 upload for testing cw721
    JSON_MSG=$(printf '{"admin":"%s","native_whitelist":[["JUNO","ujuno"]],"cw20_whitelist":[],"nft_whitelist":[]}' "$ADMIN")
    VAULT_TX=$(BINARY tx wasm instantiate "$VAULT_BASE_CODE_ID" $JSON_MSG --label "vault" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $VAULT_TX

    # == GET VAULT_CONTRACT ==
    export VAULT_CONTRACT=$(BINARY query tx $VAULT_TX --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Vault Addr: $VAULT_CONTRACT"
}
upload_vault

function upload_cw721 {
    echo "Storing CW721 contract..."
    TX721=$(BINARY tx wasm store /cw721_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo "$TX721"
    CW721_CODE_ID=$(BINARY q tx $TX721 --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $CW721_CODE_ID"
    
    echo "Instantiating CW721 contract..."
    INIT_JSON=`printf '{"name":"e2e-test","symbol":"e2e","minter":"%s"}' $KEY_ADDR`
    IMAGE_TX_UPLOAD=$(BINARY tx wasm instantiate "$CW721_CODE_ID" $INIT_JSON --label "e2e-nfts-label" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $IMAGE_TX_UPLOAD
    export CW721_CONTRACT=$(BINARY query tx $IMAGE_TX_UPLOAD --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW721_CONTRACT: $CW721_CONTRACT"        
}
upload_cw721

function mint_cw721() {
    CONTRACT_ADDR=$1
    TOKEN_ID=$2
    OWNER=$3
    TOKEN_URI=$4
    EXECUTED_MINT_JSON=`printf '{"mint":{"token_id":"%s","owner":"%s","token_uri":"%s"}}' $TOKEN_ID $OWNER $TOKEN_URI`
    TXMINT=$(BINARY tx wasm execute "$CONTRACT_ADDR" "$EXECUTED_MINT_JSON" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TXMINT
}

echo "Minting NFTs..."
mint_cw721 $CW721_CONTRACT 1 $KEY_ADDR "https://m.media-amazon.com/images/I/21IAeMeSa5L.jpg"
mint_cw721 $CW721_CONTRACT 2 $KEY_ADDR "https://m.media-amazon.com/images/I/31E1mBJT-7L.jpg"

# == Helper Functions ==
function query_contract {
    junod query wasm contract-state smart $1 $2 --output json
}
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

    tx_hash=$(BINARY tx wasm execute $CONTRACT $MESSAGE $FUNDS $JUNOD_COMMAND_ARGS | jq -r '.txhash')
    export CMD_LOG=$(BINARY query tx $tx_hash --output json | jq -r '.raw_log')    
    if [ "$SHOW_LOG" == "show_log" ]; then
        echo "raw_log: $CMD_LOG"
    fi    
}

# == ASSERTS ==
function ASSERT_EQUAL() {
    # For logs, put in quotes. 
    # If $1 is from JQ, ensure its -r and don't put in quotes
    if [ "$1" != "$2" ]; then
        echo "ERROR: $1 != $2"        
    fi
}
function ASSERT_CONTAINS {
    if [[ "$1" != *"$2"* ]]; then
        echo "ERROR: $1 does not contain $2"
        # exit 1
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
# test cw20, nft, and native (as well as tokenfactory denoms for v12)

# type_adding: 1 = Native, 2 = CW20, 3 = NFT
wasm_cmd $VAULT_CONTRACT '{"add_to_whitelist":{"type_adding":1,"to_add":["NEW","unew"]}}' ""
# add nft
wasm_cmd $VAULT_CONTRACT `printf '{"add_to_whitelist":{"type_adding":3,"to_add":["cw721","%s"]}}' $CW721_CONTRACT`


# == TEST == 
# Check that whitelist was updated
native=$(query_contract $VAULT_CONTRACT '{"get_config":{}}' | jq -r '.data.config.whitelist_native')
ASSERT_EQUAL $native '[["JUNO","ujuno"],["NEW","unew"]'
# test nft
nft=$(query_contract $VAULT_CONTRACT '{"get_config":{}}' | jq -r '.data.config.whitelist_nft')
ASSERT_EQUAL $nft `printf '[["cw721","%s"]]' $CW721_CONTRACT`



# LISTINGS
wasm_cmd $VAULT_CONTRACT '{"create_listing":{"create_msg":{"id":"vault_1","ask":{"native":[{"denom":"ujuno","amount":"10"}],"cw20":[],"nfts":[]}}}}' "1ujuno" show_log

# test listing went up correctly
listing_1=$(query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"vault_1"}}')
ASSERT_EQUAL "$listing_1" '{"data":{"creator":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl","status":"Being Prepared","for_sale":[["ujuno","1"]],"ask":[["ujuno","10"]],"expiration":"None"}}'

# Duplicate vault id, fails
wasm_cmd $VAULT_CONTRACT '{"create_listing":{"create_msg":{"id":"vault_1","ask":{"native":[{"denom":"ujuno","amount":"10"}],"cw20":[],"nfts":[]}}}}' "1ujuno" show_log
ASSERT_EQUAL "$CMD_LOG" 'failed to execute message; message index: 0: ID already taken: execute wasm contract failed'

# Finalize the listing for purchase after everything is added
wasm_cmd $VAULT_CONTRACT '{"finalize":{"listing_id":"vault_1","seconds":1000}}' "" show_log
# try to finalize again, will fail
wasm_cmd $VAULT_CONTRACT '{"finalize":{"listing_id":"vault_1","seconds":1000}}' "" show_log
ASSERT_EQUAL "$CMD_LOG" 'failed to execute message; message index: 0: Listing already finalized: execute wasm contract failed'

# Createe bucket so we can purchase the listing
wasm_cmd $VAULT_CONTRACT '{"create_bucket":{"bucket_id":"buyer_a"}}' "10ujuno" show_log
# query_contract $VAULT_CONTRACT `printf '{"get_buckets":{"bucket_owner":"%s"}}' $KEY_ADDR`

# purchase listing
wasm_cmd $VAULT_CONTRACT '{"buy_listing":{"listing_id":"vault_1","bucket_id":"buyer_a"}}' "" show_log
# check users balance changes here

# manual queries
# query_contract $VAULT_CONTRACT '{"get_config":{}}'
# query_contract $VAULT_CONTRACT '{"get_all_listings":{}}'
# query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"1"}}'
# query_contract $VAULT_CONTRACT '{"get_listings_by_owner":{"owner":"juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"}}'
# query_contract $VAULT_CONTRACT '{"get_buckets":{"bucket_owner":"juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"}}'
# query_contract $VAULT_CONTRACT '{"get_listings_for_market":{"page_num":1}}'

