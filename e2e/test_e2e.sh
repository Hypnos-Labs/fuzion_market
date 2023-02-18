#=== E2E Tests ===#
# - Creates local juno chain in docker container
# - Uploads cw20, cw721, and market contracts
# - Executes contracts following testing logic


# Functions from /e2e/helpers.sh file 
# query_contract | wasm_cmd
# mint_cw721 | create_listing_cw721 | create_bucket_cw721
# add_nft_to_listing | add_nft_to_bucket
# create_listing_cw20 | create_bucket_cw20
# add_cw20_to_listing | add_cw20_to_bucket
source ./e2e/helpers.sh

CONTAINER_NAME="fuzion_market"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
TX_FLAGS="--gas-prices 0.1$DENOM --gas-prices="0ujunox" --gas 5000000 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
export JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
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
    # docker cp ./artifacts/fuzion_market.wasm $CONTAINER_NAME:/fuzion_market.wasm
    docker cp e2e/fuzion_market.wasm $CONTAINER_NAME:/fuzion_market.wasm

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
    MARKET_UPLOAD=$($BINARY tx wasm store /fuzion_market.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $MARKET_UPLOAD
    MARKET_BASE_CODE_ID=$($BINARY q tx $MARKET_UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $MARKET_BASE_CODE_ID"

    # == INSTANTIATE ==
    ADMIN="$KEY_ADDR"
    # Do this after cw721 upload for testing cw721
    MARKET_TX=$($BINARY tx wasm instantiate "$MARKET_BASE_CODE_ID" "{}" --label "fuzion_market" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $MARKET_TX

    # == GET MARKET_CONTRACT ==
    export MARKET_CONTRACT=$($BINARY query tx $MARKET_TX --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Market Addr: $MARKET_CONTRACT"
}

function upload_cw20 {
    TYPE="CW20 Token"

    echo "Storing $TYPE contract..."
    TX=$($BINARY tx wasm store /cw20_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo "$TX"
    CW_CODE_ID=$($BINARY q tx $TX --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $CW_CODE_ID"
    
    # echo "Instantiating $TYPE contract..."
    # INIT_JSON=`printf '{"name":"e2e-test","symbol":"etoe","decimals":6,"initial_balances":[{"address":"%s","amount":"10000"}]}' $KEY_ADDR`
    # TX_UPLOAD=$($BINARY tx wasm instantiate "$CW_CODE_ID" $INIT_JSON --label "e2e-$TYPE" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $TX_UPLOAD
    # export CW20_CONTRACT=$($BINARY query tx $TX_UPLOAD --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW20_CONTRACT: $CW20_CONTRACT"  

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
    
    # echo "Instantiating CW721 contract..."
    # INIT_JSON=`printf '{"name":"e2e-test","symbol":"e2e","minter":"%s"}' $KEY_ADDR`
    # IMAGE_TX_UPLOAD=$($BINARY tx wasm instantiate "$CW721_CODE_ID" $INIT_JSON --label "e2e-nfts-label" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $IMAGE_TX_UPLOAD
    # export CW721_CONTRACT=$($BINARY query tx $IMAGE_TX_UPLOAD --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW721_CONTRACT: $CW721_CONTRACT"

    echo "Instantiating CATNFT contract..."
    INIT_JSONCAT=`printf '{"name":"e2e-cat","symbol":"cat","minter":"%s"}' $KEY_ADDR`
    IMAGE_TX_UPLOADCAT=$($BINARY tx wasm instantiate "$CW721_CODE_ID" $INIT_JSONCAT --label "e2e-cat" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $IMAGE_TX_UPLOADCAT
    export CW721_CONTRACTCAT=$($BINARY query tx $IMAGE_TX_UPLOADCAT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW721_CAT_CONTRACT: $CW721_CONTRACTCAT"

    echo "Instantiating DOGNFT contract..."
    INIT_JSONDOG=`printf '{"name":"e2e-dog","symbol":"dog","minter":"%s"}' $KEY_ADDR`
    IMAGE_TX_UPLOADDOG=$($BINARY tx wasm instantiate "$CW721_CODE_ID" $INIT_JSONDOG --label "e2e-dog" $JUNOD_COMMAND_ARGS --admin $KEY_ADDR | jq -r '.txhash') && echo $IMAGE_TX_UPLOADDOG
    export CW721_CONTRACTDOG=$($BINARY query tx $IMAGE_TX_UPLOADDOG --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW721_DOG_CONTRACT: $CW721_CONTRACTDOG"
}

# ===============
# === ASSERTS ===
# ===============
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



# =============
# === LOGIC ===
# =============

start_docker
compile_and_copy # the compile takes time for the docker container to start up

# Don't allow errors after this point
set -e

health_status
add_accounts

upload_market
upload_cw20
upload_cw721


echo "Minting NFTs..."
mint_cw721 $CW721_CONTRACTCAT 1 "$KEY_ADDR" "https://user-images.githubusercontent.com/89463679/218784962-a9a74100-8ea2-4973-91bf-623c458be9fd.png" && echo "Minted NFT CAT 1"
mint_cw721 $CW721_CONTRACTDOG 1 $KEY_ADDR "https://user-images.githubusercontent.com/89463679/218785489-7c026eb3-34b7-47f8-8e98-851462e591bc.png" && echo "Minted NFT DOG 1"

mint_cw721 $CW721_CONTRACTCAT 2 "$KEY_ADDR_TWO" "https://user-images.githubusercontent.com/89463679/218784962-a9a74100-8ea2-4973-91bf-623c458be9fd.png" && echo "Minted NFT CAT 2"
mint_cw721 $CW721_CONTRACTDOG 2 $KEY_ADDR_TWO "https://user-images.githubusercontent.com/89463679/218785489-7c026eb3-34b7-47f8-8e98-851462e591bc.png" && echo "Minted NFT DOG 2"

# == INITIAL TEST ==
# Ensure admin is correct from instantiation
# admin=$(query_contract $VAULT_CONTRACT '{"get_config":{}}' | jq -r '.data.config.admin')
# ASSERT_EQUAL "$admin" $KEY_ADDR

# Ensures we have the cw20 balance
balance=$(query_contract $CWONE_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

balance=$(query_contract $CWTWO_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

balance=$(query_contract $CWONE_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR_TWO`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

balance=$(query_contract $CWTWO_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR_TWO`)
ASSERT_EQUAL "$balance" '{"data":{"balance":"10000"}}'

# Ensures CW721 was properly minted - not needed
# token_uri=$(query_contract $CW721_CONTRACT '{"all_nft_info":{"token_id":"1"}}' | jq -r '.data.info.token_uri')
# ASSERT_EQUAL "$token_uri" "https://user-images.githubusercontent.com/89463679/218784962-a9a74100-8ea2-4973-91bf-623c458be9fd.png"

# ------ 0 amounts in ask ------- #
# - Listing with 0's in Ask should fail
# - Listing with duplicate Denoms in Ask should fail
function test_ask_failure {
    # make a listing with 2 unique but duplicate denoms, ensure the denoms are merged correctly on ask
    # failure to do this = the user can not purchase the listing, even if they sent 2ujunox
    echo "Trying to create a Listing with duplicate Native Denoms in Ask"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"1"},{"denom":"ujunox","amount":"1"}],"cw20":[],"nfts":[]}}}}' "1ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message: Cannot contain duplicate Native Tokens'

    echo "Trying to create a Listing with duplicate Cw20 Denoms in Ask"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"create_msg":{"ask":{"native":[],"cw20":[{"address":"juno1rws84uz7969aaa7pej303udhlkt3j9ca0l3egpcae98jwak9quzq8szn2l","amount":"1"},{"address":"juno1rws84uz7969aaa7pej303udhlkt3j9ca0l3egpcae98jwak9quzq8szn2l","amount":"4"}],"nfts":[]}}}}' "1ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message: Cannot contain duplicate CW20 Tokens'

    echo "Trying to create a Listing with 0 amount Native"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"0"},{"denom":"ucosm","amount":"1"}],"cw20":[],"nfts":[]}}}}' "1ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message: Cannot contain 0 value amounts'

    echo "Trying to create a Listing with 0 amount Cw20"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"create_msg":{"ask":{"native":[],"cw20":[{"address":"juno1u45shlp0q4gcckvsj06ss4xuvsu0z24a0d0vr9ce6r24pht4e5xq7q995n","amount":"1"},{"address":"juno1rws84uz7969aaa7pej303udhlkt3j9ca0l3egpcae98jwak9quzq8szn2l","amount":"0"}],"nfts":[]}}}}' "1ujunox"
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

# ------- 0 Amounts sent ------- #
# -X- Create Listing fail
# -X- Create Bucket fail
# -X- Add to Listing fail
# -X- Add to Bucket fail
function test_dupes_zeros_funds_sent {

    # 
    # Create Natives with 0
    echo "Creating listing with 0 amount Native"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"create_msg":{"ask":{"native":[{"denom":"uosmo","amount":"1"}],"cw20":[],"nfts":[]}}}}' "0ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    echo "Creating bucket with 0 amount Native"
    wasm_cmd $MARKET_CONTRACT '{"create_bucket":{}}' "0ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    #
    # Create cw20s with 0
    echo "Creating listing with 0 amount Cw20"
    create_listing_cw20 $MARKET_CONTRACT $CWONE_CONTRACT 0
    ASSERT_CONTAINS "$CMD_LOG" 'Invalid zero amount'

    echo "Creating bucket with 0 amount Cw20"
    create_bucket_cw20 $MARKET_CONTRACT $CWONE_CONTRACT 0
    ASSERT_CONTAINS "$CMD_LOG" 'Invalid zero amount'


    #
    # Creating a test listing
    echo "Creating a listing for testing"
    #create_listing_cw20 $MARKET_CONTRACT $CWONE_CONTRACT "10"
    wasm_cmd $MARKET_CONTRACT '{"create_listing":{"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"1"}],"cw20":[],"nfts":[]}}}}' "1ujunox"

    #
    # Add to Listing fail
    echo "Adding 0 Native to Listing"
    wasm_cmd $MARKET_CONTRACT '{"add_to_listing":{"listing_id":1}}' "0ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    echo "Adding 0 cw20 to Listing"
    add_cw20_to_listing $MARKET_CONTRACT $CWONE_CONTRACT "0" 1
    ASSERT_CONTAINS "$CMD_LOG" 'Invalid zero amount'

    # 
    # Create test bucket
    echo "Creating a bucket for testing"
    wasm_cmd $MARKET_CONTRACT '{"create_bucket":{}}' "1ujunox"

    #
    # Add to Bucket fail
    echo "Adding 0 Native to bucket"
    wasm_cmd $MARKET_CONTRACT '{"add_to_bucket":{"bucket_id":1}}' "0ujunox"
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    # Don't need to test cw20 again since error is at cw20 base level for any send of 0

    # Listing & Bucket ID from here after will be 2..., because of state incrementors

}


# ------ Marketplace Buy / Sell ------- #

# -X- Both Listing and Bucket have FeeDenom
# (FeeDenom is Juno by default)

function both_have_fee_denom {

    # State incrementor is 2 now, so ID's will be 2


    # export KEY_ADDR_JUNO_INITIAL=$($BINARY q bank balances $VALIDATOR_ADDR | jq -r '.[] | select(.denom == "ujunox") | .amount')

    # ================================================ #
    # KEY_ADDR creating a listing
    #   Selling: 200 JUNO, 10 CWONE, dog NFT 1
    #   Price: 200 JUNO, 20 CWTWO, cat NFT 2
    # ================================================ #
    echo "test-user creating a Listing"
    wasm_cmd $MARKET_CONTRACT "$(printf '{"create_listing":{"create_msg":{"ask":{"native":[{"denom":"ujunox","amount":"200"}],"cw20":[{"address":"%s","amount":"20"}],"nfts":[{"contract_address":"%s","token_id":"2"}]}}}}' $CWTWO_CONTRACT $CW721_CONTRACTCAT)" "200ujunox"


    # Add 10 CWONE
    echo "test-user adding 10 cwone to listing 2"
    add_cw20_to_listing $MARKET_CONTRACT $CWONE_CONTRACT "10" 2

    # Add dog NFT 1
    echo "test-user adding dog NFT 1 to listing 2"
    add_nft_to_listing $MARKET_CONTRACT $CW721_CONTRACTDOG "1" 2
    
    # KEY_ADDR finalizes listing
    echo "test-user finalizing listing"
    wasm_cmd $MARKET_CONTRACT '{"finalize":{"listing_id":2,"seconds":5000}}' ""
    # try to finalize again, will fail
    wasm_cmd $MARKET_CONTRACT '{"finalize":{"listing_id":2,"seconds":5003}}' ""
    ASSERT_CONTAINS "$CMD_LOG" 'Error Message:'

    # ================================================ #
    # KEY_ADDR_TWO creating a bucket with correct assets
    #   Correct: 200 JUNO, 20 CWTWO, cat NFT 2
    # ================================================ #

    echo "other-user creating a bucket with correct assets"
    wasm_cmd $MARKET_CONTRACT '{"create_bucket":{}}' "200ujunox" "" "$TX_FLAGS --keyring-backend test --from other-user"

    # Add 20 CWTWO
    echo "other-user adding 20 cwtwo to bucket 2"
    add_cw20_to_bucket $MARKET_CONTRACT $CWTWO_CONTRACT "20" 2 "$TX_FLAGS --keyring-backend test --from other-user"

    # Add cat NFT 2
    echo "other-user adding cat nft 2 to bucket 2"
    add_nft_to_bucket $MARKET_CONTRACT $CW721_CONTRACTCAT "2" 2 "$TX_FLAGS --keyring-backend test --from other-user"

    # ================================================ #
    # KEY_ADDR_TWO / other-user buying the Listing
    # ================================================ #

    echo "other-user buying the listing created by test-user"
    wasm_cmd $MARKET_CONTRACT '{"buy_listing":{"listing_id":2,"bucket_id":2}}' "" "$TX_FLAGS --keyring-backend test --from other-user"



    # ================================================ #
    # ================================================ #
    # Both Withdrawing
    # ================================================ #
    # ================================================ #

    # ============ 
    # test-user aka listing seller
    # ============

    # first, check balance before withdrawing
    # ujunox balance
    echo "grabbing test-user ujunox balance"
    TEST_USER_BAL=$($BINARY q bank balances $KEY_ADDR --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')
    # how to check this without manually calculating gas?
    # even if did, what about after global fee?

    # cwtwo balance
    echo "grabbing test-user cwtwo balance before withdrawing proceeds"
    TEST_USER_CWTWO_BAL=$(query_contract $CWTWO_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
    ASSERT_EQUAL "$TEST_USER_CWTWO_BAL" '{"data":{"balance":"10000"}}'

    # withdraw bucket sale proceeds
    echo "test-user withdrawing proceeds"
    wasm_cmd $MARKET_CONTRACT '{"remove_bucket":{"bucket_id":2}}' ""

    # assert test-user now has +200 JUNO?gas?, +20 CWTWO, +cat NFT 2
    echo "asserting test-user now has sale proceeds"

    # printing ujunox balances cause ? gas calcs
    TEST_USER_BAL_POST=$($BINARY q bank balances $KEY_ADDR --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')
    echo "test-user ujunox balance before withdraw: $TEST_USER_BAL ||| and after: $TEST_USER_BAL_POST"

    # cwtwo balance
    echo "test-user cwtwo balance compare check"
    TEST_USER_CWTWO_POST=$(query_contract $CWTWO_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR`)
    ASSERT_EQUAL "$TEST_USER_CWTWO_POST" '{"data":{"balance":"10020"}}'

    # nft ownership (cat nft 2)
    echo "test-user should own cat nft 2"
    CAT_TWO_OWNER=$(query_contract $CW721_CONTRACTCAT '{"owner_of":{"token_id":"2"}}' | jq -r '.data.owner')
    ASSERT_EQUAL "$CAT_TWO_OWNER" $KEY_ADDR


    # ============ 
    # other-user aka listing buyer
    # ============
    echo "now testing other-user balances"

    # get other-user ujunox balance before withdraw
    echo "grabbing other-user ujunox balance before removing purchase"
    OTHER_USER_BAL=$($BINARY q bank balances $KEY_ADDR_TWO --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')

    # withdraw purchased listing
    echo "other-user withdrawing purchased listing"
    wasm_cmd $MARKET_CONTRACT '{"withdraw_purchased":{"listing_id":2}}' "" "$TX_FLAGS --keyring-backend test --from other-user"

    # query ujunox balance after withdraw
    OTHER_USER_BAL_POST=$($BINARY q bank balances $KEY_ADDR_TWO --output json | jq -r '.balances | map(select(.denom == "ujunox")) | .[0].amount')

    # print ujunox balances
    echo "other-user ujunox balance before withdraw: $OTHER_USER_BAL ||| and after: $OTHER_USER_BAL_POST"

    # cwone balance
    echo "checking cwone balance"
    OTHER_USER_CWONE_BAL=$(query_contract $CWONE_CONTRACT `printf '{"balance":{"address":"%s"}}' $KEY_ADDR_TWO`)
    ASSERT_EQUAL $OTHER_USER_CWONE_BAL '{"data":{"balance":"10010"}}'

    # owns dog nft 1 now
    echo "other-user should own dog nft 1"
    DOG_ONE_OWNER=$(query_contract $CW721_CONTRACTDOG '{"owner_of":{"token_id":"1"}}' | jq -r '.data.owner')
    ASSERT_EQUAL "$DOG_ONE_OWNER" $KEY_ADDR_TWO

}


# running tests
test_ask_failure
test_dupes_zeros_funds_sent
both_have_fee_denom

# 1 if any of the above test failed, this way it will ensure to X the github
exit $FINAL_STATUS_CODE
