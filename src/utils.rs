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

#[cfg(test)]
mod utils_tests {


    use crate::state::*;
    use crate::utils::*;
    use cosmwasm_std::{coin, Uint128};
    use cw20::Cw20CoinVerified;
    use std::fmt::Display;

    fn here(ctx: impl Display, line: impl Display, col: impl Display) -> String {
        format!(
            "~~~~~~~~~~~~~~~~~~~ \n \n {} \n line {} | column {} \n ________________________",
            ctx, line, col
        )
    }

    fn cw20(addr: impl Into<String>, amt: u128) -> Cw20CoinVerified {
        Cw20CoinVerified {
            address: Addr::unchecked(addr),
            amount: Uint128::from(amt),
        }
    }

    fn nft(addr: impl Into<String>, id: impl Into<String>) -> Nft {
        Nft {
            contract_address: Addr::unchecked(addr),
            token_id: id.into(),
        }
    }

    fn cw20s() -> Vec<Cw20CoinVerified> {
        vec![cw20("foo", 1), cw20("bar", 2), cw20("baz", 3)]
    }

    fn nftgen() -> Vec<Nft> {
        vec![nft("boredcats", "30"), nft("dogs", "31"), nft("sharks", "32")]
    }


    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Testing calc_fee_coin math
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    
    #[test]
    fn fee_denom_not_in_gbal() {

        // FeeDenom is Juno, GenericBalance doesn't contain JUNO
        // Output of calc_fee_coin should be Ok((None, Initial GenericBalance Unchanged))

        let native = vec![coin(200, "uatom"), coin(300, "uosmo")];
        // ujunox
        let juno_fee_denom = FeeDenom::JUNO;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&juno_fee_denom, &gbal).expect(&here("y", line!(), column!()));

        //fee_coin should be none
        if fee_coin.is_some() {
            panic!("{}", here("Fee coin should be none", line!(), column!()));
        }

        // new_gbal should be == old gbal
        genbal_cmp(&new_gbal, &gbal).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });

        // uusdcx
        let usdc_fee_denom = FeeDenom::USDC;

        let (fee_coinx, new_gbalx) = calc_fee_coin(&usdc_fee_denom, &gbal).expect(&here("y", line!(), column!()));

        //fee_coin should be none
        if fee_coinx.is_some() {
            panic!("{}", here("Fee coin should be none", line!(), column!()));
        }

        // new_gbalx should be == old gbal
        genbal_cmp(&new_gbalx, &gbal).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });
    }

    #[test]
    fn fee_denom_juno_basic() {

        // FeeDenom is Juno, GenericBalance contains JUNO
        // Output of calc_fee_coin should be (0.5% of JUNO, Initial GenericBalance - 0.5% of JUNO)

        let native = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(1000, "ujunox")];

        // ujunox
        let juno_fee_denom = FeeDenom::JUNO;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&juno_fee_denom, &gbal).expect(&here("y", line!(), column!()));

        //fee_coin should be Some(5 ujunox)
        assert_eq!(
            Some(coin(5, "ujunox")),
            fee_coin,
            "Juno fee incorrect: {}", line!()
        );

        // new_gbal should have everything the same, except 995 ujunox
        let nativex = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(995, "ujunox")];
        let test = GenericBalance {
            native: nativex,
            cw20: cw20s(),
            nfts: nftgen()
        };
        genbal_cmp(&new_gbal, &test).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });

    }

    #[test]
    fn fee_denom_usdc_basic() {

        // FeeDenom is Usdc, GenericBalance contains Usdc
        // Output of calc_fee_coin should be (0.5% of Usdc, Initial GenericBalance - 0.5% of Usdc)

        let native = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(1000, "uusdcx")];

        let usdc_fee_denom = FeeDenom::USDC;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&usdc_fee_denom, &gbal).expect(&here("y", line!(), column!()));

        //fee_coin should be Some(5 ujunox)
        assert_eq!(
            Some(coin(5, "uusdcx")),
            fee_coin,
            "USDC fee incorrect: {}", line!()
        );

        // new_gbal should have everything the same, except 995 ujunox
        let nativex = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(995, "uusdcx")];
        let test = GenericBalance {
            native: nativex,
            cw20: cw20s(),
            nfts: nftgen()
        };
        genbal_cmp(&new_gbal, &test).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });

    }

    #[test]
    fn fee_denom_juno_div() {

        // 999 * 5 / 1000 = 4.995
        // fee coin should be 4
        // gbal should have 999 - 4 = 995

        let native = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(999, "ujunox")];

        // ujunox
        let juno_fee_denom = FeeDenom::JUNO;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&juno_fee_denom, &gbal).expect(&here("y", line!(), column!()));

        //fee_coin should be Some(4 ujunox)
        assert_eq!(
            Some(coin(4, "ujunox")),
            fee_coin,
            "Juno fee incorrect: {}", line!()
        );

        // new_gbal should have everything the same, except 995 ujunox
        let nativex = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(995, "ujunox")];
        let test = GenericBalance {
            native: nativex,
            cw20: cw20s(),
            nfts: nftgen()
        };
        genbal_cmp(&new_gbal, &test).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });
    }

    #[test]
    fn fee_denom_usdc_div() {

        // 999 * 5 / 1000 = 4.995
        // fee coin should be 4
        // gbal should have 999 - 4 = 995

        let native = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(999, "uusdcx")];

        // ujunox
        let usdc_fee_denom = FeeDenom::USDC;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&usdc_fee_denom, &gbal).expect(&here("y", line!(), column!()));

        //fee_coin should be Some(4 uusdcx)
        assert_eq!(
            Some(coin(4, "uusdcx")),
            fee_coin,
            "USDC fee incorrect: {}", line!()
        );

        // new_gbal should have everything the same, except 995 uusdcx
        let nativex = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(995, "uusdcx")];
        let test = GenericBalance {
            native: nativex,
            cw20: cw20s(),
            nfts: nftgen()
        };
        genbal_cmp(&new_gbal, &test).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });
    }

    #[test]
    fn smallest_possible_juno() {

        // 200 * 5 / 1000 = 1
        // Smallest possible amount that includes a fee should be 200
        // anything under this should have no fee
        let native = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(200, "ujunox")];

        // ujunox
        let juno_fee_denom = FeeDenom::JUNO;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&juno_fee_denom, &gbal).expect(&here("y", line!(), column!()));

        //fee_coin should be Some(1 ujunox)
        assert_eq!(
            Some(coin(1, "ujunox")),
            fee_coin,
            "Juno fee incorrect: {}", line!()
        );

        // new_gbal should have everything the same, except 199 ujunox
        let nativex = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(199, "ujunox")];
        let test = GenericBalance {
            native: nativex,
            cw20: cw20s(),
            nfts: nftgen()
        };
        genbal_cmp(&new_gbal, &test).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });

    }

    #[test]
    fn smallest_possible_usdc() {

        // 200 * 5 / 1000 = 1
        // Smallest possible amount that includes a fee should be 200
        // anything under this should have no fee
        let native = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(200, "uusdcx")];

        // ujunox
        let usdc_fee_denom = FeeDenom::USDC;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&usdc_fee_denom, &gbal).expect(&here("y", line!(), column!()));

        //fee_coin should be Some(1 ujunox)
        assert_eq!(
            Some(coin(1, "uusdcx")),
            fee_coin,
            "USDC fee incorrect: {}", line!()
        );

        // new_gbal should have everything the same, except 199 ujunox
        let nativex = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(199, "uusdcx")];
        let test = GenericBalance {
            native: nativex,
            cw20: cw20s(),
            nfts: nftgen()
        };
        genbal_cmp(&new_gbal, &test).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });

    }

    #[test]
    fn too_small_juno() {

        // 199 * 5 / 1000 = 0.995
        // 0.995 should get rounded down to 0,
        // return should be Ok((None, Old gbal))
        
        let native = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(199, "ujunox")];

        // ujunox
        let juno_fee_denom = FeeDenom::JUNO;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&juno_fee_denom, &gbal).expect(&here("y", line!(), column!()));


        //fee_coin should be none
        if fee_coin.is_some() {
            panic!("{}", here("Fee coin should be none", line!(), column!()));
        }

        // new_gbal should be == old gbal
        genbal_cmp(&new_gbal, &gbal).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });
    }

    #[test]
    fn too_small_usdc() {

        // 199 * 5 / 1000 = 0.995
        // 0.995 should get rounded down to 0,
        // return should be Ok((None, Old gbal))
        
        let native = vec![coin(200, "uatom"), coin(300, "uosmo"), coin(199, "uusdcx")];

        // uusdcx
        let usdc_fee_denom = FeeDenom::USDC;

        let gbal: GenericBalance = GenericBalance { 
            native,
            cw20: cw20s(),
            nfts: nftgen()
        };


        let (fee_coin, new_gbal) = calc_fee_coin(&usdc_fee_denom, &gbal).expect(&here("y", line!(), column!()));


        //fee_coin should be none
        if fee_coin.is_some() {
            panic!("{}", here("Fee coin should be none", line!(), column!()));
        }

        // new_gbal should be == old gbal
        genbal_cmp(&new_gbal, &gbal).unwrap_or_else(|_| { panic!("{}", here(
            "Should be equal",
            line!(),
            column!(),
        )) });
    }


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

