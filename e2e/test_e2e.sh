# Test script for Juno Smart Contracts (By @Reecepbcups)
# ./github/workflows/e2e.yml
#
# sh ./e2e/test_e2e.sh

CONTAINER_NAME="juno_cw_vaults"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
TX_FLAGS="--gas-prices 0.1$DENOM --gas-prices="0ujunox" --gas 5000000 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"

function stop_docker {
    docker kill $CONTAINER_NAME
    docker volume rm -f junod_data
}

function start_docker {
    IMAGE_TAG=${2:-"v11.0.3"}
    BLOCK_GAS_LIMIT=${GAS_LIMIT:-100000000} # mirrors mainnet

    echo "Building $IMAGE_TAG"
    echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

    stop_docker    

    # run junod docker
    docker run --rm -d --name $CONTAINER_NAME \
        -e STAKE_TOKEN=$DENOM \
        -e GAS_LIMIT="$GAS_LIMIT" \
        -e UNSAFE_CORS=true \
        -p 1317:1317 -p 26656:26656 -p 26657:26657 \
        --mount type=volume,source=junod_data,target=/root \
        ghcr.io/cosmoscontracts/juno:$IMAGE_TAG /opt/setup_and_run.sh $KEY_ADDR    
}

function compile_and_copy {
    # compile vaults contract here
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/rust-optimizer:0.12.11

    # copy wasm to docker container
    docker cp ./artifacts/juno_vaults.wasm $CONTAINER_NAME:/juno_vaults.wasm

    # copy helper contracts to container
    docker cp e2e/cw20_base.wasm $CONTAINER_NAME:/cw20_base.wasm
    docker cp e2e/cw721_base.wasm $CONTAINER_NAME:/cw721_base.wasm
}

start_docker
# the compile takes time for the docker container to start up
compile_and_copy
# docker exec -it $CONTAINER_NAME ls /

# == Helper Functions ==
function query_contract {
    $BINARY query wasm contract-state smart $1 $2 --output json
}

function mint_cw721 {
    CONTRACT_ADDR=$1
    TOKEN_ID=$2
    OWNER=$3
    TOKEN_URI=$4
    EXECUTED_MINT_JSON=`printf '{"mint":{"token_id":"%s","owner":"%s","token_uri":"%s"}}' $TOKEN_ID $OWNER $TOKEN_URI`
    TXMINT=$($BINARY tx wasm execute "$CONTRACT_ADDR" "$EXECUTED_MINT_JSON" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TXMINT
}

function wasm_cmd {
    CONTRACT=$1
    MESSAGE=$2
    FUNDS=$3
    SHOW_LOG=${4:dont_show}
    echo "EXECUTE $MESSAGE on $CONTRACT"

    # if length of funds is 0, then no funds are sent
    if [ -z "$FUNDS" ]; then
        FUNDS=""
    else
        FUNDS="--amount $FUNDS"
        echo "FUNDS: $FUNDS"
    fi

    tx_hash=$($BINARY tx wasm execute $CONTRACT $MESSAGE $FUNDS $JUNOD_COMMAND_ARGS | jq -r '.txhash')
    export CMD_LOG=$($BINARY query tx $tx_hash --output json | jq -r '.raw_log')    
    if [ "$SHOW_LOG" == "show_log" ]; then
        echo -e "raw_log: $CMD_LOG\n================================\n"
    fi    
}

# == ASSERTS ==
FINAL_STATUS_CODE=0

function ASSERT_EQUAL {
    # For logs, put in quotes. 
    # If $1 is from JQ, ensure its -r and don't put in quotes
    if [ "$1" != "$2" ]; then        
        echo "ERROR: $1 != $2" 1>&2
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS"
    fi
}

function ASSERT_CONTAINS {
    if [[ "$1" != *"$2"* ]]; then
        echo "ERROR: $1 does not contain $2" 1>&2        
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS"
    fi
}


# validator addr
VALIDATOR_ADDR=$($BINARY keys show validator --address) && echo "Validator address: $VALIDATOR_ADDR"

BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR) && echo "Pre-store balance: $BALANCE_1"

echo "Address to deploy contracts: $KEY_ADDR"
echo "JUNOD_COMMAND_ARGS: $JUNOD_COMMAND_ARGS"
# === COPY ALL ABOVE TO SET ENVIROMENT UP LOCALLY ====

# == errors from this point on are no bueno ==
set -e

# === LOGIC ===
# provision juno default user i.e. juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl
echo "decorate bright ozone fork gallery riot bus exhaust worth way bone indoor calm squirrel merry zero scheme cotton until shop any excess stage laundry" | $BINARY keys add test-user --recover --keyring-backend test
# juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk
echo "wealth flavor believe regret funny network recall kiss grape useless pepper cram hint member few certain unveil rather brick bargain curious require crowd raise" | $BINARY keys add other-user --recover --keyring-backend test

# send some 10 junox funds to the user
$BINARY tx bank send test-user juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk 10000000ujunox $JUNOD_COMMAND_ARGS

# check funds where sent
other_balance=$($BINARY q bank balances juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk --output json)
ASSERT_EQUAL "$other_balance" '{"balances":[{"denom":"ujunox","amount":"10000000"}],"pagination":{"next_key":null,"total":"0"}}'

function upload_vault {
    # == UPLOAD VAULT ==
    echo "Storing Vault contract..."
    VAULT_UPLOAD=$($BINARY tx wasm store /juno_vaults.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $VAULT_UPLOAD
    VAULT_BASE_CODE_ID=$($BINARY q tx $VAULT_UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $VAULT_BASE_CODE_ID"

    # == INSTANTIATE ==
    ADMIN="$KEY_ADDR"
    # Do this after cw721 upload for testing cw721
    JSON_MSG=$(printf '{"admin":"%s"}' "$ADMIN")
    VAULT_TX=$($BINARY tx wasm instantiate "$VAULT_BASE_CODE_ID" $JSON_MSG --label "vault" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $VAULT_TX

    # == GET VAULT_CONTRACT ==
    export VAULT_CONTRACT=$($BINARY query tx $VAULT_TX --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Vault Addr: $VAULT_CONTRACT"
}

function upload_cw20 {
    TYPE="CW20 Token"

    echo "Storing $TYPE contract..."
    TX=$($BINARY tx wasm store /cw20_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo "$TX"
    CW_CODE_ID=$($BINARY q tx $TX --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $CW_CODE_ID"
    
    echo "Instantiating $TYPE contract..."
    INIT_JSON=`printf '{"name":"e2e-test","symbol":"etoe","decimals":6,"initial_balances":[{"address":"%s","amount":"10000"}]}' $KEY_ADDR`
    TX_UPLOAD=$($BINARY tx wasm instantiate "$CW_CODE_ID" $INIT_JSON --label "e2e-$TYPE" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $TX_UPLOAD
    export CW20_CONTRACT=$($BINARY query tx $TX_UPLOAD --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW20_CONTRACT: $CW20_CONTRACT"        
}

function upload_cw721 {
    echo "Storing CW721 contract..."
    TX721=$($BINARY tx wasm store /cw721_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo "$TX721"
    CW721_CODE_ID=$($BINARY q tx $TX721 --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $CW721_CODE_ID"
    
    echo "Instantiating CW721 contract..."
    INIT_JSON=`printf '{"name":"e2e-test","symbol":"e2e","minter":"%s"}' $KEY_ADDR`
    IMAGE_TX_UPLOAD=$($BINARY tx wasm instantiate "$CW721_CODE_ID" $INIT_JSON --label "e2e-nfts-label" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $IMAGE_TX_UPLOAD
    export CW721_CONTRACT=$($BINARY query tx $IMAGE_TX_UPLOAD --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW721_CONTRACT: $CW721_CONTRACT"        
}

upload_vault
upload_cw20
upload_cw721


echo "Minting NFTs..."
mint_cw721 $CW721_CONTRACT 1 "$KEY_ADDR" "https://m.media-amazon.com/images/I/21IAeMeSa5L.jpg"
mint_cw721 $CW721_CONTRACT 2 $KEY_ADDR "https://m.media-amazon.com/images/I/31E1mBJT-7L.jpg"


# == INITIAL TEST ==
# Ensures we have the cw20 balance
balance=$(query_contract $CW20_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

# Ensures CW721 was properly minted - not needed
token_uri=$(query_contract $CW721_CONTRACT '{"all_nft_info":{"token_id":"1"}}' | jq -r '.data.info.token_uri')
ASSERT_EQUAL "$token_uri" "https://m.media-amazon.com/images/I/21IAeMeSa5L.jpg"

# Ensure admin is correct from instantiation
admin=$(query_contract $VAULT_CONTRACT '{"get_config":{}}' | jq -r '.data.config.admin')
ASSERT_EQUAL "$admin" $KEY_ADDR

# === LISTINGS TEST ===

# Selling 10ucosm for 5ujunox
wasm_cmd $VAULT_CONTRACT '{"create_listing":{"create_msg":{"id":"vault_1","ask":{"native":[{"denom":"ujunox","amount":"5"}],"cw20":[],"nfts":[]}}}}' "10ucosm" show_log

# Ensure listing went up correctly
listing_1=$(query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"vault_1"}}')
ASSERT_EQUAL "$listing_1" '{"data":{"creator":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl","status":"Being Prepared","for_sale":[["ucosm","10"]],"ask":[["ujunox","5"]],"expiration":"None"}}'

# Duplicate vault id, expect fail
wasm_cmd $VAULT_CONTRACT '{"create_listing":{"create_msg":{"id":"vault_1","ask":{"native":[{"denom":"ujunox","amount":"1"}],"cw20":[],"nfts":[]}}}}' "1ujunox"
ASSERT_CONTAINS "$CMD_LOG" 'ID already taken'


# CW721 NFTS
function send_nft_to_listing {
    INTERATING_CONTRACT=$1
    CW721_CONTRACT_ADDR=$2
    TOKEN_ID=$3
    LISTING_ID=$4
    
    NFT_LISTING_BASE64=`printf '{"add_to_listing_cw721":{"listing_id":"%s"}}' $LISTING_ID | base64 -w 0`    
    SEND_NFT_JSON=`printf '{"send_nft":{"contract":"%s","token_id":"%s","msg":"%s"}}' $INTERATING_CONTRACT $TOKEN_ID $NFT_LISTING_BASE64`        

    wasm_cmd $CW721_CONTRACT_ADDR "$SEND_NFT_JSON" "" show_log
}

echo "Sending NFT id 1 to the listing"
send_nft_to_listing $VAULT_CONTRACT $CW721_CONTRACT "1" "vault_1"
query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"vault_1"}}'

# owner should now be the VAULT_CONTRACT after sending
owner_of_nft=$(query_contract $CW721_CONTRACT '{"all_nft_info":{"token_id":"1"}}' | jq -r '.data.access.owner')
ASSERT_EQUAL "$owner_of_nft" "$VAULT_CONTRACT"


function send_cw20_to_listing {
    INTERATING_CONTRACT=$1
    CW20_CONTRACT_ADDR=$2
    AMOUNT=$3
    LISTING_ID=$4
    
    LISTING_BASE64=`printf '{"add_funds_to_sale_cw20":{"listing_id":"%s"}}' $LISTING_ID | base64 -w 0`               
    SEND_TOKEN_JSON=`printf '{"send":{"contract":"%s","amount":"%s","msg":"%s"}}' $INTERATING_CONTRACT $AMOUNT $LISTING_BASE64`        

    wasm_cmd $CW20_CONTRACT_ADDR "$SEND_TOKEN_JSON" "" show_log
}

# Send 1 coin to the listing
send_cw20_to_listing $VAULT_CONTRACT $CW20_CONTRACT "20" "vault_1"

# Ensure the CW20 token & CW721 is now apart of the listing
# todo: this will fail if the order of the array changes given there is no difference between cw20 and cw721 in it.
listing_values=$(query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"vault_1"}}' | jq -r '.data.for_sale')
ASSERT_EQUAL $listing_values `printf '[["ucosm","10"],["%s","20"],["%s":"1"]]' $CW20_CONTRACT $CW721_CONTRACT`


# Finalize the listing for purchase after everything is added
wasm_cmd $VAULT_CONTRACT '{"finalize":{"listing_id":"vault_1","seconds":5000}}' "" show_log
# try to finalize again, will fail
wasm_cmd $VAULT_CONTRACT '{"finalize":{"listing_id":"vault_1","seconds":100}}' ""
ASSERT_CONTAINS "$CMD_LOG" 'Listing already finalized'

# Create bucket so we can purchase the listing
wasm_cmd $VAULT_CONTRACT '{"create_bucket":{"bucket_id":"buyer_1"}}' "5ujunox" show_log

# purchase listing
wasm_cmd $VAULT_CONTRACT '{"buy_listing":{"listing_id":"vault_1","bucket_id":"buyer_1"}}' "" show_log
# check users balance changes here after we  execute_withdraw_purchased
# query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"vault_1"}}' <- ensure it is closed, but I feel when we buy it should auto transfer? Why not?

# withdraw purchase
wasm_cmd $VAULT_CONTRACT '{"withdraw_purchased":{"listing_id":"vault_1"}}' "" show_log

# ensure listings are empty now
listings=$(query_contract $VAULT_CONTRACT '{"get_all_listings":{}}' | jq -r '.data.listings')
ASSERT_EQUAL "$listings" '[]'

# Check they got their tokens (native, cw20, cw721)

# 1 if any of the above test failed
exit $FINAL_STATUS_CODE

# manual queries
# query_contract $VAULT_CONTRACT '{"get_config":{}}'
# query_contract $VAULT_CONTRACT '{"get_all_listings":{}}'
# query_contract $VAULT_CONTRACT '{"get_listing_info":{"listing_id":"1"}}'
# query_contract $VAULT_CONTRACT '{"get_listings_by_owner":{"owner":"juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"}}'
# query_contract $VAULT_CONTRACT '{"get_buckets":{"bucket_owner":"juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"}}'
# query_contract $VAULT_CONTRACT '{"get_listings_for_market":{"page_num":1}}'