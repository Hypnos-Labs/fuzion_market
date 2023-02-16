use crate::utils_imports::*;

// Actual community pool on mainnet
//const COMMUNITY_POOL: &str = "juno1jv65s3grqf6v6jl3dp4t6c9t9rk99cd83d88wr";

/// Accepts 2 parameters:
/// - `to`: A **user** address (cannot be a smart contract address)
/// - `balance`: A GenericBalance object containing any number of Native, CW20, or CW721s
///
/// Returns `StdError` on Binary serialization issues
///
/// Otherwise returns `Ok(Vec<CosmosMsg>)`, where each `CosmosMsg` is sending the items
/// from within the `GenericBalance` to the `to` User Address
pub fn send_tokens_cosmos(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<CosmosMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<CosmosMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![CosmosMsg::from(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.clone(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            // Only works if recipient is User Address, doesn't work for DAO / Contracts
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = CosmosMsg::from(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();

    msgs.extend(cw20_msgs?);

    let nft_balance = &balance.nfts;
    let nft_msgs: StdResult<Vec<CosmosMsg<Empty>>> = nft_balance
        .iter()
        .map(|n| {
            let msg = Cw721ExecuteMsg::TransferNft {
                recipient: to.into(),
                token_id: n.token_id.clone(),
            };
            let exec = CosmosMsg::from(WasmMsg::Execute {
                contract_addr: n.contract_address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();

    msgs.extend(nft_msgs?);

    Ok(msgs)
}

/// Accepts current FeeDenom & GenericBalance, returns one of the following
///
///
/// **If FeeDenom is not in GenericBalance || fifty_pips floored is zero**
/// - Returns Fee Coin as None +
/// - Returns Balance unchanged
/// - `Ok((None, GenericBalance))`
///
///
/// **If FeeDenom is in Balance && fifty_pips floored is not zero**
/// - Returns FeeCoin as fifty_pips of Fee Denom in Balance
/// - Returns Balance as Balance - FeeCoin
/// - Returns Ok((Some(Coin), Balance))
///
///
/// **Returns StdError on int overflow**
pub fn calc_fee_coin(
    fee_type: &FeeDenom,
    balance: &GenericBalance,
) -> StdResult<(Option<Coin>, GenericBalance)> {
    // Get the current fee denom to check for
    let fee_denom = fee_type.value();

    // Find the fee denom in balance
    let fee_in_balance = balance.native.iter().find(|n| n.denom == fee_denom);

    // Calculate Fee Amount & return
    match fee_in_balance {
        // If fee_denom not found, return (None, balance)
        None => Ok((None, balance.to_owned())),

        // If fee_denom found, calculate fee coin
        Some(fee) => {
            // Calc 0.5% of fee_denom found
            let fifty_pips = fee.amount.multiply_ratio(5_u128, 1000_u128);

            // small amounts (like 1ujuno) will be 0, so return None
            if fifty_pips.is_zero() {
                return Ok((None, balance.to_owned()));
            }

            // Create Fee Coin
            let fee_coin = coin(fifty_pips.u128(), fee_denom.clone());

            // Subtract fee amount from the fee coin found in balance
            // Sub fee amount from balance_fee coin amount
            let amount_sub_fee = fee.amount.checked_sub(fifty_pips)?;

            // Create GenericBalance with fee amount removed
            let balance_with_fee_removed = {
                let mut x = balance.clone();
                x.native.retain(|n| n.denom != fee_denom);
                x.native.append(&mut coins(amount_sub_fee.u128(), fee_denom));
                x
            };

            // Return (Fee, Balance_minus_fee)
            Ok((Some(fee_coin), balance_with_fee_removed))
        }
    }
}


// encode a protobuf into a cosmos message
// Inspired by https://github.com/alice-ltd/smart-contracts/blob/master/contracts/alice_terra_token/src/execute.rs#L73-L76
pub fn proto_encode<M: prost::Message>(msg: M, type_url: String) -> StdResult<CosmosMsg> {
    let mut bytes = Vec::new();
    prost::Message::encode(&msg, &mut bytes)
        .map_err(|_e| StdError::generic_err("Message encoding must be infallible"))?;
    Ok(cosmwasm_std::CosmosMsg::<cosmwasm_std::Empty>::Stargate {
        type_url,
        value: cosmwasm_std::Binary(bytes),
    })
}

// Accepts a `GenericBalance` and calculates the fee to be paid, based on the current fee denom
// pub fn calc_fee(current_fee: FeeDenom, balance: &GenericBalance) -> StdResult<Option<(CosmosMsg, GenericBalance)>> {
//     // Get the current fee denom to check for
//     let fee_denom = current_fee.value();
//     // Find the fee denom in balance
//     let fee_in_balance = balance.native.iter().find(|n| n.denom == fee_denom);
//     // If balance DOES NOT contain fee_denom, return Ok(None)
//     // If balance DOES contain fee_denom, calculate 0.5% of the denom in the balance,
//     // Create CosmosMsg sending that to the Community Pool,
//     // and return this CosmosMsg + a generic balance with the fee removed for the user
//     if let Some(balance_fees) = fee_in_balance {
//         // 0.5% = amount * 5 / 1000
//         // will be floored (fee may be < 0.5%, amount to user will not be < 99.5%)
//         let fifty_pips = balance_fees.amount.multiply_ratio(5_u128, 1000_u128);
//         // small amounts (like 1ujuno) will be 0
//         if fifty_pips.is_zero() {
//             return Ok(None);
//         }
//         // Fee Msg sending fifty_pips to community pool
//         let fee_msg: CosmosMsg<Empty> = CosmosMsg::from(BankMsg::Send {
//             to_address: COMMUNITY_POOL.to_string(),
//             amount: coins(fifty_pips.u128(), fee_denom.clone()),
//         });
//         // Sub fee amount from balance_fee coin amount
//         let amount_sub_fee = balance_fees.amount.checked_sub(fifty_pips)?;
//         // Create GenericBalance with fee amount removed from fee_denom
//         let balance_with_fee_removed = {
//             let mut x = balance.clone();
//             x.native.retain(|n| n.denom != fee_denom);
//             x.native.append(&mut coins(amount_sub_fee.u128(), fee_denom));
//             x
//         };
//         Ok(Some((fee_msg, balance_with_fee_removed)))
//     } else {
//         Ok(None)
//     }
// }
