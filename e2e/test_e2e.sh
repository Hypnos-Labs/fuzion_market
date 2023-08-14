# ================================================== #
# ==================== E2E Tests =================== #
# ========= Original by @Reecepbcups =============== #
# ================================================== #
# - Creates local juno chain in docker container
# - Uploads cw20, cw721, and market contracts
# - Executes contracts following testing logic
# ================================================== #

# Import helper functions for interacting with contract
source ./e2e/helpers.sh

CONTAINER_NAME="fuzion_market"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
TX_FLAGS="--gas-prices=0.1$DENOM --gas=auto --gas-adjustment=1.5 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
export JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
export JUNOD_COMMAND_ARGS_OTHER="$TX_FLAGS --from other-user"
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"
export KEY_ADDR_TWO="juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk"


# ===================
# === Docker Init ===
# ===================
function stop_docker {
    docker kill $CONTAINER_NAME
    docker rm $CONTAINER_NAME
    docker volume rm -f junod_data
}

function start_docker {
    IMAGE_TAG=${2:-"15.0.0"}
    BLOCK_GAS_LIMIT=${GAS_LIMIT:-100000000} # mirrors mainnet

    echo "Building $IMAGE_TAG"
    echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

    stop_docker    

    # run junod docker
    docker run --rm -d --name $CONTAINER_NAME \
        -e STAKE_TOKEN=$DENOM \
        -e GAS_LIMIT="$GAS_LIMIT" \
        -e TIMEOUT_COMMIT=500ms \
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
      cosmwasm/workspace-optimizer:0.14.0

    # copy market contract to docker container
    docker cp ./artifacts/marketplace.wasm $CONTAINER_NAME:/marketplace.wasm
    # docker cp e2e/fuzion_market.wasm $CONTAINER_NAME:/fuzion_market.wasm

    # copy royalty contract to docker container
    docker cp ./artifacts/royalty.wasm $CONTAINER_NAME:/royalty.wasm

    # copy helper contracts to container
    docker cp e2e/cw20_base.wasm $CONTAINER_NAME:/cw20_base.wasm
    docker cp e2e/cw721_base.wasm $CONTAINER_NAME:/cw721_base.wasm
}

function health_status {
    # validator addr
    VALIDATOR_ADDR=$($BINARY keys show validator --address) && echo "Validator address: $VALIDATOR_ADDR"

    BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR) && echo "Pre-store balance: $BALANCE_1"
    # export KEY_ADDR_JUNO_INITIAL=$($BINARY q bank balances $VALIDATOR_ADDR | jq -r '.[] | select(.denom == "ujunox") | .amount')

    echo "Address to deploy contracts: $KEY_ADDR"
    echo "JUNOD_COMMAND_ARGS: $JUNOD_COMMAND_ARGS"
}

# ========================
# === Contract Uploads ===
# ========================
function upload_market {
    # == UPLOAD market ==
    echo "Storing Market contract..."
    MARKET_UPLOAD=$($BINARY tx wasm store /marketplace.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $MARKET_UPLOAD
    MARKET_BASE_CODE_ID=$($BINARY q tx $MARKET_UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Market Code Id: $MARKET_BASE_CODE_ID"

    # == UPLOAD Royalty ==
    echo "Storing Royalty contract..."
    ROYALTY_UPLOAD=$($BINARY tx wasm store /royalty.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $ROYALTY_UPLOAD
    ROYALTY_CODE_ID=$($BINARY q tx $ROYALTY_UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Royalty Code Id: $ROYALTY_CODE_ID"

    # == INSTANTIATE ==
    ADMIN="$KEY_ADDR"
    MARKET_INIT=`printf '{"royalty_code_id":%d}' $ROYALTY_CODE_ID`
    # Do this after cw721 upload for testing cw721
    MARKET_TX=$($BINARY tx wasm instantiate "$MARKET_BASE_CODE_ID" "$MARKET_INIT" --label "fuzion_market" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $MARKET_TX

    # == GET MARKET_CONTRACT ==
    export MARKET_CONTRACT=$($BINARY query tx $MARKET_TX --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Market Addr: $MARKET_CONTRACT"

    # == Get Royalty Contract address ==
    export ROYALTY_CONTRACT=$(query_contract $MARKET_CONTRACT '{"get_royalty_addr":{}}') && echo "Royalty Addr: $ROYALTY_CONTRACT"

}

function upload_cw20 {
    TYPE="CW20 Token"

    echo "Storing $TYPE contract..."
    TX=$($BINARY tx wasm store /cw20_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo "$TX"
    CW_CODE_ID=$($BINARY q tx $TX --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $CW_CODE_ID"
    
    echo "Instantiating CWONE contract..."
    INIT_JSONONE=`printf '{"name":"cw-one","symbol":"cwone","decimals":6,"initial_balances":[{"address":"%s","amount":"10000"},{"address":"%s","amount":"10000"}]}' $KEY_ADDR $KEY_ADDR_TWO`
    TX_UPLOADONE=$($BINARY tx wasm instantiate "$CW_CODE_ID" $INIT_JSONONE --label "e2e-cwone" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $TX_UPLOADONE
    export CWONE_CONTRACT=$($BINARY query tx $TX_UPLOADONE --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CWONE_CONTRACT: $CWONE_CONTRACT"

    echo "Instantiating CWTWO contract..."
    INIT_JSONTWO=`printf '{"name":"cw-two","symbol":"cwtwo","decimals":6,"initial_balances":[{"address":"%s","amount":"10000"},{"address":"%s","amount":"10000"}]}' $KEY_ADDR $KEY_ADDR_TWO`
    TX_UPLOADTWO=$($BINARY tx wasm instantiate "$CW_CODE_ID" $INIT_JSONTWO --label "e2e-cwtwo" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $TX_UPLOADTWO
    export CWTWO_CONTRACT=$($BINARY query tx $TX_UPLOADTWO --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CWTWO_CONTRACT: $CWTWO_CONTRACT"      
}

function upload_cw721 {
    echo "Storing CW721 contract..."
    TX721=$($BINARY tx wasm store /cw721_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo "$TX721"
    CW721_CODE_ID=$($BINARY q tx $TX721 --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $CW721_CODE_ID"
    
    echo "Instantiating CATNFT contract..."
    INIT_JSONCAT=`printf '{"name":"e2e-cat","symbol":"cat","minter":"%s"}' $KEY_ADDR`
    IMAGE_TX_UPLOADCAT=$($BINARY tx wasm instantiate "$CW721_CODE_ID" $INIT_JSONCAT --label "e2e-cat" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $IMAGE_TX_UPLOADCAT
    export CW721_CONTRACTCAT=$($BINARY query tx $IMAGE_TX_UPLOADCAT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW721_CAT_CONTRACT: $CW721_CONTRACTCAT"

    echo "Instantiating DOGNFT contract..."
    INIT_JSONDOG=`printf '{"name":"e2e-dog","symbol":"dog","minter":"%s"}' $KEY_ADDR`
    IMAGE_TX_UPLOADDOG=$($BINARY tx wasm instantiate "$CW721_CODE_ID" $INIT_JSONDOG --label "e2e-dog" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $IMAGE_TX_UPLOADDOG
    export CW721_CONTRACTDOG=$($BINARY query tx $IMAGE_TX_UPLOADDOG --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW721_DOG_CONTRACT: $CW721_CONTRACTDOG"
}

# Registering cat NFT for 100 bps royalties (1%)
function register_royalties {
    echo "Registering cat for 100 bps royalties..."
    REG_CAT_ROYALTY=`printf '{"register":{"nft_contract":"%s","payout_addr":"%s","bps":100}}' $CW721_CONTRACTCAT $KEY_ADDR_TWO`

    wasm_cmd $ROYALTY_CONTRACT $REG_CAT_ROYALTY "" show_log

}

# ===============
# === ASSERTS ===
# ===============
FINAL_STATUS_CODE=0

function CHECK_FEE {
    BEFORE=$1
    AFTER=$2
    WITHDRAWN=$3

    # after - before needs to be == 99.5% of withdrawn
    DIFF=$(echo "$AFTER-$BEFORE" | bc -l)
    EXPECTED=$(echo "0.995*$WITHDRAWN" | bc -l)

    if (( $(echo "$DIFF != $EXPECTED" | bc -l) )); then        
        echo "ERROR: Fee amount not correct: $DIFF != $EXPECTED" 1>&2
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS: Fee amount successfully removed: $DIFF == $EXPECTED"
    fi
}

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

function add_accounts {
    # provision juno default user i.e. juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl
    echo "decorate bright ozone fork gallery riot bus exhaust worth way bone indoor calm squirrel merry zero scheme cotton until shop any excess stage laundry" | $BINARY keys add test-user --recover --keyring-backend test
    # juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk
    echo "wealth flavor believe regret funny network recall kiss grape useless pepper cram hint member few certain unveil rather brick bargain curious require crowd raise" | $BINARY keys add other-user --recover --keyring-backend test

    # send some 10 junox funds to the user
    $BINARY tx bank send test-user juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk 10000000ujunox $JUNOD_COMMAND_ARGS

    # check funds where sent
    other_balance=$($BINARY q bank balances juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk --output json)
    ASSERT_EQUAL "$other_balance" '{"balances":[{"denom":"ujunox","amount":"10000000"}],"pagination":{"next_key":null,"total":"0"}}'
}

# === COPY ALL ABOVE TO SET ENVIROMENT UP LOCALLY ====

# ===================================================================== #
# =============================== Setup =============================== #
# ===================================================================== #

start_docker
compile_and_copy

# Don't allow errors after this point
set -e

health_status
add_accounts

upload_market
upload_cw20
upload_cw721
register_royalties


echo "Minting NFTs..."
mint_cw721 $CW721_CONTRACTCAT 1 $KEY_ADDR "https://user-images.githubusercontent.com/89463679/218784962-a9a74100-8ea2-4973-91bf-623c458be9fd.png" && echo "Minted NFT CAT 1"
mint_cw721 $CW721_CONTRACTDOG 1 $KEY_ADDR "https://user-images.githubusercontent.com/89463679/218785489-7c026eb3-34b7-47f8-8e98-851462e591bc.png" && echo "Minted NFT DOG 1"

mint_cw721 $CW721_CONTRACTCAT 2 $KEY_ADDR_TWO "https://user-images.githubusercontent.com/89463679/218784962-a9a74100-8ea2-4973-91bf-623c458be9fd.png" && echo "Minted NFT CAT 2"
mint_cw721 $CW721_CONTRACTDOG 2 $KEY_ADDR_TWO "https://user-images.githubusercontent.com/89463679/218785489-7c026eb3-34b7-47f8-8e98-851462e591bc.png" && echo "Minted NFT DOG 2"



# Ensures CW721 was properly minted - not needed
# token_uri=$(query_contract $CW721_CONTRACT '{"all_nft_info":{"token_id":"1"}}' | jq -r '.data.info.token_uri')
# ASSERT_EQUAL "$token_uri" "https://user-images.githubusercontent.com/89463679/218784962-a9a74100-8ea2-4973-91bf-623c458be9fd.png"

# Ensures we have the cw20 balance
balance=$(query_contract $CWONE_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

balance=$(query_contract $CWTWO_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

balance=$(query_contract $CWONE_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR_TWO`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

balance=$(query_contract $CWTWO_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR_TWO`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

# ============================================================= #
# ======================= Logic Tests ========================= #
# ============================================================= #

# 

# Listings cannot have 0 amounts or duplicates in their Ask Price
# [X] Listing with 0's in Ask should fail
# [X] Listing with duplicate Denoms in Ask should fail
function test_ask_failure {
    # make a listing with 2 unique but duplicate denoms, ensure the denoms are merged correctly on ask
    # failure to do this = the user can not purchase the listing, even if they sent 2ujunox
    echoe "Trying to create a Listing with duplicate Native Denoms in Ask"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":1,"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"1"},{"denom":"ujunox","amount":"1"}],"cw20":[],"nfts":[]}}}}' "1ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message: Cannot contain duplicate Native Tokens'

    echoe "Trying to create a Listing with duplicate Cw20 Denoms in Ask"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":2,"create_msg":{"ask":{"native":[],"cw20":[{"address":"juno1rws84uz7969aaa7pej303udhlkt3j9ca0l3egpcae98jwak9quzq8szn2l","amount":"1"},{"address":"juno1rws84uz7969aaa7pej303udhlkt3j9ca0l3egpcae98jwak9quzq8szn2l","amount":"4"}],"nfts":[]}}}}' "1ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message: Cannot contain duplicate CW20 Tokens'

    echoe "Trying to create a Listing with 0 amount Native in Ask"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":3,"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"0"},{"denom":"ucosm","amount":"1"}],"cw20":[],"nfts":[]}}}}' "1ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message: Cannot contain 0 value amounts'

    echoe "Trying to create a Listing with 0 amount Cw20 in Ask"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":4,"create_msg":{"ask":{"native":[],"cw20":[{"address":"juno1u45shlp0q4gcckvsj06ss4xuvsu0z24a0d0vr9ce6r24pht4e5xq7q995n","amount":"1"},{"address":"juno1rws84uz7969aaa7pej303udhlkt3j9ca0l3egpcae98jwak9quzq8szn2l","amount":"0"}],"nfts":[]}}}}' "1ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message: Cannot contain 0 value amounts'

    # # finalize
    # wasm_cmd $MARKET_CONTRACT '{"finalize":{"listing_id":"vault_combine","seconds":5000}}' "" show_log

    # # buy the listing to keep future test clean
    # wasm_cmd $MARKET_CONTRACT '{"create_bucket":{"bucket_id":"buyer_com"}}' "2ujunox" show_log
    # wasm_cmd $MARKET_CONTRACT '{"buy_listing":{"listing_id":"vault_combine","bucket_id":"buyer_com"}}' "" show_log 
    # wasm_cmd $MARKET_CONTRACT '{"withdraw_purchased":{"listing_id":"vault_combine"}}' "" dont_show

    # # ensure the listing was removed        
    # listings=$(query_contract $MARKET_CONTRACT '{"get_all_listings":{}}' --output json)       
    # ASSERT_EQUAL $listings '{"data":{"listings":[]}}'

}

# Cannot create a Listing by sending 0 amounts
# [X] Create Listing fail
# [X] Create Bucket fail
# [X] Add to Listing fail
# [X] Add to Bucket fail
function test_dupes_zeros_funds_sent {

    # Create Natives with 0
    echoe "Creating listing with 0 amount Native"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":5,"create_msg":{"ask":{"native":[{"denom":"uosmo","amount":"1"}],"cw20":[],"nfts":[]}}}}' "0ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    echoe "Creating bucket with 0 amount Native"
    wasm_cmd $MARKET_CONTRACT '{"create_bucket":{"bucket_id":1}}' "0ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    # Create cw20s with 0
    echoe "Creating listing with 0 amount Cw20"
    create_listing_cw20 $MARKET_CONTRACT $CWONE_CONTRACT "0" 6
    ASSERT_CONTAINS "$CMD_LOG" 'Invalid zero amount'

    echoe "Creating bucket with 0 amount Cw20"
    create_bucket_cw20 $MARKET_CONTRACT $CWONE_CONTRACT "0" 7
    ASSERT_CONTAINS "$CMD_LOG" 'Invalid zero amount'

    # Creating a test listing
    echoe "Creating a listing for testing"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":1,"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"1"}],"cw20":[],"nfts":[]}}}}' "1ujunox"

    # Add to Listing fail
    echoe "Adding 0 Native to Listing"
    wasm_cmd $MARKET_CONTRACT '{"add_to_listing":{"listing_id":1}}' "0ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    echoe "Adding 0 cw20 to Listing"
    add_cw20_to_listing $MARKET_CONTRACT $CWONE_CONTRACT "0" 1
    ASSERT_CONTAINS "$CMD_LOG" 'Invalid zero amount'

    # Create test bucket
    echoe "Creating a bucket for testing"
    wasm_cmd $MARKET_CONTRACT '{"create_bucket":{"bucket_id":1}}' "1ujunox"

    # Add to Bucket fail
    echoe "Adding 0 Native to bucket"
    wasm_cmd $MARKET_CONTRACT '{"add_to_bucket":{"bucket_id":1}}' "0ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    # Don't need to test cw20 again since error is at cw20_base level for any send of 0

    # Listing & Bucket ID from here after will be 2

}


# Testing that a correctly structured trade executes, and correct fee amount is removed
# [X] Both Listing and Bucket have FeeDenom (JUNO by default)
function both_have_fee_denom {
    # State incrementor is 2 now, so ID's will be 2
    # export KEY_ADDR_JUNO_INITIAL=$($BINARY q bank balances $VALIDATOR_ADDR | jq -r '.[] | select(.denom == "ujunox") | .amount')

    # ================================================ #
    # ==== test-user / KEY_ADDR creating Listing ===== #
    # ================================================ #
    #   Selling: 200 JUNO, 10 CWONE, dog NFT 1
    #   Price: 200 JUNO, 20 CWTWO, cat NFT 2
    # ================================================ #

    # test-user / KEY_ADDR creates Listing 2
    echoe "test-user creating Listing 2"
    wasm_cmd $MARKET_CONTRACT "$(printf '{"create_listing":{"listing_id":2,"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"200"}],"cw20":[{"address":"%s","amount":"20"}],"nfts":[{"contract_address":"%s","token_id":"2"}]}}}}' $CWTWO_CONTRACT $CW721_CONTRACTCAT)" "200ujunox" show_log

    # test-user adds 10 cwone to Listing 2
    echoe "test-user adding 10 cwone to listing 2"
    add_cw20_to_listing $MARKET_CONTRACT $CWONE_CONTRACT "10" 2

    # test-user adds Dog NFT #1 to Listing 2
    echoe "test-user adding dog NFT 1 to listing 2"
    add_nft_to_listing $MARKET_CONTRACT $CW721_CONTRACTDOG "1" 2
    
    # test-user finalizes Listing 2
    echoe "test-user finalizing listing"
    wasm_cmd $MARKET_CONTRACT '{"finalize":{"listing_id":2,"seconds":5000}}' ""
    # try to finalize again, will fail
    wasm_cmd $MARKET_CONTRACT '{"finalize":{"listing_id":2,"seconds":5003}}' ""
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    # ================================================ #
    # === other-user / KEY_ADDR_TWO create Bucket ==== #
    # ================================================ #
    #  200 JUNO, 20 CWTWO, cat NFT 2 (same as price)
    # ================================================ #

    # other-user / KEY_ADDR_TWO creates Bucket 2
    echoe "other-user creating a bucket with correct assets"
    wasm_cmd_other $MARKET_CONTRACT '{"create_bucket":{"bucket_id":2}}' "200ujunox"

    # other-user adds 20 cwtwo to Bucket 2
    echoe "other-user adding 20 cwtwo to bucket 2"
    add_cw20_to_bucket_other $MARKET_CONTRACT $CWTWO_CONTRACT "20" 2

    # other-user adds Cat NFT #2 to Bucket 2
    echoe "other-user adding cat nft 2 to bucket 2"
    add_nft_to_bucket_other $MARKET_CONTRACT $CW721_CONTRACTCAT "2" 2

    # ================================================ #
    # ================================================ #
    # KEY_ADDR_TWO / other-user buying Listing 2
    # ================================================ #

    # COUNT=$(query_contract $MARKET_CONTRACT '{"get_all_listings":{}}' --output json)
    # echoe $COUNT
    # COUNTB=$(query_contract $MARKET_CONTRACT "$(printf '{"get_buckets":{"bucket_owner":"%s"}}' $KEY_ADDR_TWO)" --output json)
    # echoe $COUNTB

    echoe "other-user buying the listing created by test-user"
    wasm_cmd_other $MARKET_CONTRACT '{"buy_listing":{"listing_id":2,"bucket_id":2}}' ""

    # ================================================ #
    # ================================================ #
    # Both users withdrawing
    # ================================================ #

    # =============================
    # test-user (listing seller)
    # ============================= 

    # first, check balance before withdrawing
    # ujunox balance
    echoe "grabbing test-user ujunox balance"
    TEST_USER_BAL=$($BINARY q bank balances $KEY_ADDR --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')
    # how to check this without manually calculating gas?
    # even if did, what about after global fee?

    # cwtwo balance
    echoe "grabbing test-user cwtwo balance before withdrawing proceeds"
    TEST_USER_CWTWO_BAL=$(query_contract $CWTWO_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
    ASSERT_EQUAL "$TEST_USER_CWTWO_BAL" '{"data":{"balance":"10000"}}'

    # withdraw bucket sale proceeds
    echoe "test-user withdrawing proceeds"
    wasm_cmd $MARKET_CONTRACT '{"remove_bucket":{"bucket_id":2}}' ""

    # assert test-user now has +200 JUNO?gas?, +20 CWTWO, +cat NFT 2
    echoe "asserting test-user now has sale proceeds"

    # printing ujunox balances cause ? gas calcs
    TEST_USER_BAL_POST=$($BINARY q bank balances $KEY_ADDR --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')
    echoe "test-user ujunox balance before withdraw: $TEST_USER_BAL ||| and after: $TEST_USER_BAL_POST"

    CHECK_FEE $TEST_USER_BAL $TEST_USER_BAL_POST 200

    # cwtwo balance
    echoe "test-user cwtwo balance compare check"
    TEST_USER_CWTWO_POST=$(query_contract $CWTWO_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
    ASSERT_EQUAL "$TEST_USER_CWTWO_POST" '{"data":{"balance":"10020"}}'

    # nft ownership (cat nft 2)
    echoe "test-user should own cat nft 2"
    CAT_TWO_OWNER=$(query_contract $CW721_CONTRACTCAT '{"owner_of":{"token_id":"2"}}' | jq -r '.data.owner')
    ASSERT_EQUAL "$CAT_TWO_OWNER" $KEY_ADDR


    # ==================================
    # other-user (listing buyer)
    # ==================================
    echoe "now testing other-user balances"

    # get other-user ujunox balance before withdraw
    echoe "grabbing other-user ujunox balance before removing purchase"
    OTHER_USER_BAL=$($BINARY q bank balances $KEY_ADDR_TWO --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')

    # withdraw purchased listing
    echoe "other-user withdrawing purchased listing"
    wasm_cmd_other $MARKET_CONTRACT '{"withdraw_purchased":{"listing_id":2}}' ""

    # query ujunox balance after withdraw
    OTHER_USER_BAL_POST=$($BINARY q bank balances $KEY_ADDR_TWO --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')

    # print ujunox balances
    echoe "other-user ujunox balance before withdraw: $OTHER_USER_BAL ||| and after: $OTHER_USER_BAL_POST"
    
    CHECK_FEE $OTHER_USER_BAL $OTHER_USER_BAL_POST 200

    # cwone balance
    echoe "checking cwone balance"
    OTHER_USER_CWONE_BAL=$(query_contract $CWONE_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR_TWO`)
    ASSERT_EQUAL $OTHER_USER_CWONE_BAL '{"data":{"balance":"10010"}}'

    # owns dog nft 1 now
    echoe "other-user should own dog nft 1"
    DOG_ONE_OWNER=$(query_contract $CW721_CONTRACTDOG '{"owner_of":{"token_id":"1"}}' | jq -r '.data.owner')
    ASSERT_EQUAL "$DOG_ONE_OWNER" $KEY_ADDR_TWO

}


# Testing what happens if a Listing/bucket has a large amount of
# different assets when withdrawing for out of gas errors
# for out of gas issues
# NOTE: Currently there is a hardcoded limit of 25 different assets (nfts + cw20s + natives)
# So as long as 25 NFTs & 25 CW20s succeeds
# [X] 100 NFTs failure <===
# [X]  75 NFTs failure <===
# [X]  50 NFTs success 
# [x]  Hardcoded 25 Asset limit
function big_sale {
    # State incrementor is 3 now, so ID's will be 3

    # Create super long ask price and minting 100 NFTs for each user
    nfts=""
    for ((i=3;i<=27;i++))
    do
        if [[ $i -eq 3 ]]; then
            nfts="{\"contract_address\":\"$CW721_CONTRACTDOG\",\"token_id\":\"$i\"}"
        else
            nfts="$nfts,{\"contract_address\":\"$CW721_CONTRACTDOG\",\"token_id\":\"$i\"}"
        fi

        mint_cw721 $CW721_CONTRACTCAT "$i" $KEY_ADDR "https://google.com"
        mint_cw721 $CW721_CONTRACTDOG "$i" $KEY_ADDR_TWO "https://google.com"
    done

    # test-user creates listing 3 with 200 ujunox and Dog #3 - #38 in ask
    # will be selling 200ujunox and Cat NFT #3 - 38
    wasm_cmd $MARKET_CONTRACT "$(printf '{"create_listing":{"listing_id":3,"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"200"}],"cw20":[],"nfts":[%s]}}}}' "$nfts")" "200ujunox" show_log

    # other-user creates bucket 3 with 200 ujunox
    wasm_cmd_other $MARKET_CONTRACT '{"create_bucket":{"bucket_id":3}}' "200ujunox" show_log

    # test-user adds Cat #3 - 38 to listing 3
    # other-user adds Dog #3 - 38 to bucket 3
    echoe "adding 25 NFTs to listing 3 and bucket 3"
    for ((i=3;i<=27;i++))
    do
        # test-user adds cat $i to listing 3
        add_nft_to_listing $MARKET_CONTRACT $CW721_CONTRACTCAT "$i" 3

        # other-user adds dog $i to bucket 3
        add_nft_to_bucket_other $MARKET_CONTRACT $CW721_CONTRACTDOG "$i" 3
    done


    # test-user finalizes Listing 3
    echoe "test-user finalizing listing 3"
    wasm_cmd $MARKET_CONTRACT '{"finalize":{"listing_id":3,"seconds":50000}}' "" show_log

    # other-user buying listing 3 with bucket 3
    echoe "other-user buying the listing 3 with bucket 3"
    wasm_cmd_other $MARKET_CONTRACT '{"buy_listing":{"listing_id":3,"bucket_id":3}}' "" show_log



    # ================================================ #
    # ================================================ #
    # Both users withdrawing
    # ================================================ #

    # =============================
    # test-user (listing seller)
    # ============================= 

    # first, check balance before withdrawing
    # ujunox balance
    echoe "grabbing test-user ujunox balance"
    TEST_USER_BAL=$($BINARY q bank balances $KEY_ADDR --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')

    # withdraw bucket sale proceeds
    echoe "test-user withdrawing proceeds of bucket 3"
    wasm_cmd $MARKET_CONTRACT '{"remove_bucket":{"bucket_id":3}}' "" show_log

    # assert test-user now has dog NFT #3 - 103
    echoe "asserting test-user now has sale proceeds"

    TEST_USER_BAL_POST=$($BINARY q bank balances $KEY_ADDR --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')
    echoe "test-user ujunox balance before withdraw: $TEST_USER_BAL ||| and after: $TEST_USER_BAL_POST"

    CHECK_FEE $TEST_USER_BAL $TEST_USER_BAL_POST 200

    echoe "test-user should own dog NFT 3 - 35"
    DOG_X_OWNER=$(query_contract $CW721_CONTRACTDOG '{"owner_of":{"token_id":"3"}}' | jq -r '.data.owner')
    ASSERT_EQUAL "$DOG_X_OWNER" $KEY_ADDR

    DOG_XX_OWNER=$(query_contract $CW721_CONTRACTDOG '{"owner_of":{"token_id":"35"}}' | jq -r '.data.owner')
    ASSERT_EQUAL "$DOG_XX_OWNER" $KEY_ADDR


    # ==================================
    # other-user (listing buyer)
    # ==================================
    echoe "now testing other-user balances"

    # get other-user ujunox balance before withdraw
    echoe "grabbing other-user ujunox balance before removing purchase"
    OTHER_USER_BAL=$($BINARY q bank balances $KEY_ADDR_TWO --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')

    # withdraw purchased listing
    echoe "other-user withdrawing purchased listing"
    wasm_cmd_other $MARKET_CONTRACT '{"withdraw_purchased":{"listing_id":3}}' "" show_log

    # query ujunox balance after withdraw
    OTHER_USER_BAL_POST=$($BINARY q bank balances $KEY_ADDR_TWO --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')

    # print ujunox balances
    echoe "other-user ujunox balance before withdraw: $OTHER_USER_BAL ||| and after: $OTHER_USER_BAL_POST"
    
    CHECK_FEE $OTHER_USER_BAL $OTHER_USER_BAL_POST 200

    echoe "other-user should own cat NFT 3 - 35"
    CAT_X_OWNER=$(query_contract $CW721_CONTRACTCAT '{"owner_of":{"token_id":"3"}}' | jq -r '.data.owner')
    ASSERT_EQUAL "$CAT_X_OWNER" $KEY_ADDR_TWO

    CAT_XX_OWNER=$(query_contract $CW721_CONTRACTCAT '{"owner_of":{"token_id":"35"}}' | jq -r '.data.owner')
    ASSERT_EQUAL "$CAT_XX_OWNER" $KEY_ADDR_TWO

}


# Checking that previously used ID's (even ones that have been removed & deleted)
# cannot be used again
function check_prev_ids {
    echoe "Creating Listing with prev used id #1 "
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":1,"create_msg":{"ask":{"native":[{"denom":"uosmo","amount":"1"}],"cw20":[],"nfts":[]}}}}' "5ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    echoe "Creating bucket with prev used id #1"
    wasm_cmd $MARKET_CONTRACT '{"create_bucket":{"bucket_id":1}}' "5ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    echoe "Creating Listing with prev used id #2"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":2,"create_msg":{"ask":{"native":[{"denom":"uosmo","amount":"1"}],"cw20":[],"nfts":[]}}}}' "5ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    echoe "Creating bucket with prev used id #2"
    wasm_cmd $MARKET_CONTRACT '{"create_bucket":{"bucket_id":2}}' "5ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'
}

function check_max_id {
    echoe "Creating Listing with too high ID (9007199254740990)"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"listing_id":9007199254740990,"create_msg":{"ask":{"native":[{"denom":"uosmo","amount":"1"}],"cw20":[],"nfts":[]}}}}' "5ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    echoe "Creating bucket with too high ID (9007199254740990)"
    wasm_cmd $MARKET_CONTRACT '{"create_bucket":{"bucket_id":9007199254740990}}' "5ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

}


# running tests
test_ask_failure
test_dupes_zeros_funds_sent
both_have_fee_denom
big_sale
check_prev_ids
check_max_id

# 1 if any of the above test failed, this way it will ensure to X the github
exit $FINAL_STATUS_CODE
