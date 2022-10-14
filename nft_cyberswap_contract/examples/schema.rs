//use std::env::current_dir;
//use std::fs::create_dir_all;
//use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
//use cyberswap::msg::*;
//use cyberswap::state::*;

use cosmwasm_schema::write_api;
use cyberswap_nft::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}

//fn main() {
//    let mut out_dir = current_dir().unwrap();
//    out_dir.push("schema");
//    create_dir_all(&out_dir).unwrap();
//    remove_schemas(&out_dir).unwrap();
//
//    // MSGs
//    export_schema(&schema_for!(InstantiateMsg), &out_dir);
//    export_schema(&schema_for!(ExecuteMsg), &out_dir);
//    export_schema(&schema_for!(ReceiveMsg), &out_dir);
//    export_schema(&schema_for!(QueryMsg), &out_dir);
//
//    // STATE items
//    export_schema(&schema_for!(Config), &out_dir);
//    export_schema(&schema_for!(GenericBalance), &out_dir);
//    export_schema(&schema_for!(Listing), &out_dir);
//    export_schema(&schema_for!(Status), &out_dir);
//    export_schema(&schema_for!(Bucket), &out_dir);
//
//    // Create Listing Msg
//    export_schema(&schema_for!(CreateListingMsg), &out_dir);
//
//    // QUERY Responses
//    export_schema(&schema_for!(AdminResponse), &out_dir);
//    export_schema(&schema_for!(MultiListingResponse), &out_dir);
//    export_schema(&schema_for!(ListingInfoResponse), &out_dir);
//    export_schema(&schema_for!(GetBucketsResponse), &out_dir);
//}
//