#[cfg(not(feature = "library"))]
use crate::{
    error::ContractError, 
    state::REGISTRY
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr
};
use cw2::set_contract_version;
use royalties::{
    //DenomType, 
    RoyaltyInfo
};
use royalties::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// Registration can only be updated every 100 blocks
pub const COOLDOWN_BLOCKS: u64 = 100u64;
// Max royalty bps is 300 (3%) | Min royalty bps is 10 (0.1%)
const MAX_BPS: u64 = 300u64;
const MIN_BPS: u64 = 10u64;

const CONTRACT_NAME: &str = "crates.io:royalty";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("Instantiate", "royalty"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register { 
            nft_contract,
            payout_addr, 
            bps 
        } => register_royalties(deps, env, info, nft_contract, payout_addr, bps),
        ExecuteMsg::Update { 
            nft_contract,
            new_payout_addr, 
            new_bps
        } => update_royalties(deps, env, info, nft_contract, new_payout_addr, new_bps),
        ExecuteMsg::Remove { 
            nft_contract 
        } => remove_registration(deps, env, info, nft_contract),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn register_royalties(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft_contract: String,
    payout_addr: String,
    bps: u64,
) -> Result<Response, ContractError> {

    // Validate bps amount
    if bps > MAX_BPS || bps < MIN_BPS {
        return Err(ContractError::GenericError("Max royalty amount is 300 bps (3%) | Min royalty amount is 10 bps (0.1%)".to_string()));
    }

    // Validate addresses
    let valid_payout_addr = deps.api.addr_validate(&payout_addr)?;
    let valid_nft_contract = deps.api.addr_validate(&nft_contract)?;

    // Verify that sender is NFT contract admin
    let z = deps.querier.query_wasm_contract_info(valid_nft_contract.as_str())?;

    // If sender is not nft_contract admin or there is no admin, action is unauthorized
    if !z.admin.clone().map_or(false, |admin| admin == info.sender.to_string()) {
        let msg = format!("Unauthorized | Sender ({:#?}) isn't admin ({:#?})", info.sender.to_string(), z.admin);
        return Err(ContractError::GenericError(msg));
    }

    // Error if entry exists
    if REGISTRY.has(deps.storage, &valid_nft_contract) {
        return Err(ContractError::GenericError("Already registered | Call update for modifications".to_string()));
    }

    // Save registration
    REGISTRY.save(
        deps.storage,
        &valid_nft_contract,
        &RoyaltyInfo {
            last_updated: env.block.height,
            bps,
            payout_addr: valid_payout_addr
        }
    )?;

    Ok(Response::new()
            .add_attribute("Registered", valid_nft_contract.as_str()))

}

#[allow(clippy::too_many_arguments)]
pub fn update_royalties(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft_contract: String,
    new_payout_addr: Option<String>,
    new_bps: Option<u64>,
) -> Result<Response, ContractError> {

    // Validate nft_contract addr
    let valid_nft_contract = deps.api.addr_validate(&nft_contract)?;

    // Verify that sender is NFT contract admin
    let z = deps.querier.query_wasm_contract_info(valid_nft_contract.as_str())?;

    // If sender is not nft_contract admin or there is no admin, action is unauthorized
    if !z.admin.clone().map_or(false, |admin| admin == info.sender.to_string()) {
        let msg = format!("Unauthorized | Sender ({:#?}) isn't admin ({:#?})", info.sender.to_string(), z.admin);
        return Err(ContractError::GenericError(msg));
    }

    // Load existing entry from storage, error if non existent
    let entry: RoyaltyInfo = REGISTRY.load(deps.storage, &valid_nft_contract)
        .map_err(|_e| ContractError::GenericError("Contract not registered".to_string()))?;

    // Validate cooldown has passed 
    if entry.last_updated.saturating_add(COOLDOWN_BLOCKS) > env.block.height {
        return Err(ContractError::GenericError("Cooldown not yet complete".to_string()));
    }

    // Validate new BPS
    let bps = match new_bps {
        Some(b) => {
            if b > MAX_BPS || b < MIN_BPS {
                Err(ContractError::GenericError("Max royalty amount is 300 bps (3%) | Min royalty amount is 10 bps (0.1%)".to_string()))
            } else {
                Ok(b)
            }
        },
        None => Ok(entry.bps)
    }?;

    // Validate new payout address
    let payout_addr = match new_payout_addr {
        Some(addr) => deps.api.addr_validate(addr.as_str()),
        None => Ok(entry.payout_addr)
    }?;

    // Validate new denoms (add native validation when cosmwasm_1_3 is live)
    // let denom_one = match new_denom_one {
    //     Some(DenomType::Cw20(ref addr)) => {
    //         deps.api.addr_validate(&addr)?;
    //         DenomType::Cw20(addr.clone())
    //     },
    //     Some(DenomType::Native(n)) => DenomType::Native(n),
    //     None => entry.denom_one
    // };

    // Update entry
    REGISTRY.save(
        deps.storage,
        &valid_nft_contract,
        &RoyaltyInfo {
            last_updated: env.block.height,
            bps,
            payout_addr
        }
    )?;

    Ok(Response::new().add_attribute("Updated", valid_nft_contract.as_str()))
}


pub fn remove_registration(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft_contract: String
) -> Result<Response, ContractError> {

    // Validate nft_contract addr
    let valid_nft_contract = deps.api.addr_validate(&nft_contract)?;

    // Verify that sender is NFT contract admin
    let z = deps.querier.query_wasm_contract_info(valid_nft_contract.as_str())?;

    // If sender is not nft_contract admin or there is no admin, action is unauthorized
    if !z.admin.map_or(false, |admin| admin == info.sender.to_string()) {
        return Err(ContractError::GenericError("Unauthorized".to_string()));
    }

    // Check that cooldown has passed, prevents de-register/re-register exploit
    let entry: RoyaltyInfo = REGISTRY.load(deps.storage, &valid_nft_contract)
        .map_err(|_e| ContractError::GenericError("Contract not registered".to_string()))?;
 
    if entry.last_updated.saturating_add(COOLDOWN_BLOCKS) > env.block.height {
        return Err(ContractError::GenericError("Cooldown not yet complete".to_string()));
    }

    // Remove
    REGISTRY.remove(deps.storage, &valid_nft_contract);

    Ok(Response::new().add_attribute("Removed", valid_nft_contract.as_str()))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::RoyaltyInfoSingle { 
            nft_contract 
        } => to_binary(&get_single(deps, nft_contract)?),
        QueryMsg::RoyaltyInfoMulti { 
            nft_contracts 
        } => to_binary(&get_multi(deps, nft_contracts)?)
    }
}

pub fn get_multi(
    deps: Deps,
    nft_contracts: Vec<String>
) -> StdResult<Vec<Option<RoyaltyInfo>>> {

    if nft_contracts.is_empty() {
        return Err(cosmwasm_std::StdError::GenericErr { msg: "No contracts found".to_string() });
    }

    // Shouldn't need addr_validate but verify in e2e/integration tests
    let infos: Vec<Option<RoyaltyInfo>> = nft_contracts
        .iter()
        .map(|contract| {
            REGISTRY.may_load(deps.storage, &Addr::unchecked(contract))
        }).collect::<Result<Vec<Option<RoyaltyInfo>>, cosmwasm_std::StdError>>()?;

    Ok(infos)

}

pub fn get_single(
    deps: Deps,
    nft_contract: String
) -> StdResult<Option<RoyaltyInfo>> {

    REGISTRY.may_load(deps.storage, &Addr::unchecked(nft_contract))
}

#[cfg(test)]
#[allow(dead_code, unused)]
mod tests {
    use super::*;
    use cosmwasm_std::{Binary, Uint128};

    #[test]
    fn bps() {

        // 50%
        let z = cosmwasm_std::Decimal::bps(5000);

        //assert!(false, "{}", z);

    }
}
