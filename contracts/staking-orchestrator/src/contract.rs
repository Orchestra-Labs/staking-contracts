#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use std::collections::HashMap;
use std::hash::Hash;

use crate::error::ContractError;
use crate::msg::{AllTokensStakedBalanceAtHeightResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StakingContractByDenomResponse};
use crate::state::{RegisteredContract, STAKING_CONTRACTS};
use cosmwasm_std::{to_json_binary, Binary, DenomUnit, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, WasmMsg};
use cw2::set_contract_version;
use cw_ownable::get_ownership;
use cw_utils::{parse_instantiate_response_data, Duration};

pub(crate) const CONTRACT_NAME: &str = "crates.io:symphony-staking-orchestrator";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_STAKING_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<Empty>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg.owner.unwrap_or(info.sender.to_string());
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;

    Ok(
        Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("owner", owner)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut,
               env: Env,
               info: MessageInfo,
               msg: ExecuteMsg) -> Result<Response<Empty>, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => execute_update_owner(deps, info, env, action),
        ExecuteMsg::CreateStakingContract { code_id, denom_unit, unbonding_period, owner } => {
            execute_create_staking_contract(deps, env, info, code_id, denom_unit, unbonding_period, owner)
        }
    }
}

pub fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    action: cw_ownable::Action,
) -> Result<Response<Empty>, ContractError> {
    let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::default().add_attributes(ownership.into_attributes()))
}

pub fn execute_create_staking_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    denom_unit: DenomUnit,
    unbonding_period: Option<Duration>,
    owner: Option<String>,
) -> Result<Response<Empty>, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let selected_owner = deps.api.addr_validate(
        owner.as_deref().unwrap_or(env.contract.address.as_str())
    )?;

    let msg = symphony_interfaces::staking::InstantiateMsg {
        owner: Some(selected_owner.to_string()),
        denom_unit: denom_unit.clone(),
        unbonding_period,
    };

    let init_msg = WasmMsg::Instantiate {
        admin: Some(selected_owner.to_string()),
        code_id,
        msg: to_json_binary(&msg.clone())?,
        funds: info.funds,
        label: format!("{} staking contract", denom_unit.denom),
    };

    let msg = SubMsg::reply_on_success(init_msg, INSTANTIATE_STAKING_REPLY_ID);

    Ok(
        Response::new()
            .add_attribute("action", "create_staking_contract")
            .add_attribute("denom", denom_unit.denom)
            .add_attribute("owner", selected_owner)
            .add_attribute("code_id", format!("{}", code_id))
            .add_submessage(msg)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_json_binary(&get_ownership(deps.storage)?),
        QueryMsg::StakingContractByDenom { denom } =>
            to_json_binary(&query_staking_contract_by_denom(deps, denom)?),
        QueryMsg::AllTokensStakedBalanceAtHeight { address, height } =>
            to_json_binary(&query_all_tokens_staked_balance_at_height(deps, address, height)?),
    }
}

pub fn query_staking_contract_by_denom(deps: Deps, denom: String) -> StdResult<StakingContractByDenomResponse> {
    let registered_contract = STAKING_CONTRACTS.load(deps.storage, &denom)?;

    Ok(StakingContractByDenomResponse {
        denom,
        registered_contract,
    })
}

pub fn query_all_tokens_staked_balance_at_height(
    deps: Deps,
    address: String,
    height: Option<u64>
) -> StdResult<AllTokensStakedBalanceAtHeightResponse> {
    let mut tokens_staked_balance = HashMap::new();

    let contracts = STAKING_CONTRACTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<Vec<_>>>()?;

    for contract in contracts {
        let result: symphony_interfaces::staking::StakedBalanceAtHeightResponse = deps.querier.query_wasm_smart(
            contract.address.clone(),
            &native_staking::msg::QueryMsg::StakedBalanceAtHeight {
                address: address.clone(),
                height,
            },
        )?;

        tokens_staked_balance.insert(contract.token.denom, result);
    }



    Ok(AllTokensStakedBalanceAtHeightResponse {
        tokens_staked_balance,
    })
}

fn query_staking_contract(deps: Deps, address: String) -> StdResult<RegisteredContract> {
    let result: native_staking::msg::ConfigResponse = deps.querier.query_wasm_smart(
        address.clone(),
        &native_staking::msg::QueryMsg::Config {},
    )?;

    let contract = RegisteredContract {
        address,
        token: result.staking_token,
    };

    Ok(contract)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        INSTANTIATE_STAKING_REPLY_ID => {
            handle_instantiate_staking_reply(deps, env, msg)
        },
        _ => Err(StdError::generic_err("Invalid reply ID")),
    }
}

fn decode_and_handle_binary_data(deps: DepsMut, bin: &Binary) -> StdResult<Response> {
    let decoded = parse_instantiate_response_data(&bin)
        .map_err(|e| StdError::generic_err(format!("parsing submsg response: {}", e)))?;

    let contract_config = query_staking_contract(
        deps.as_ref(),
        decoded.contract_address.clone()
    )?;

    let contract = RegisteredContract {
        address: decoded.contract_address.clone(),
        token: contract_config.token.clone(),
    };

    STAKING_CONTRACTS.save(deps.storage, &contract_config.token.denom, &contract)?;

    Ok(
        Response::new()
            .add_attribute("action", "create_staking_contract")
            .add_attribute("denom", contract.token.denom)
            .add_attribute("address", decoded.contract_address)
    )
}

fn handle_instantiate_staking_reply(
    deps: DepsMut,
    _env: Env,
    msg: Reply,
) -> StdResult<Response> {
    match msg.result.into_result() {
        Err(e) => {
            Err(StdError::generic_err(format!("SubMsg failed: {}", e)))
        }
        Ok(sub_msg) => {
            match sub_msg.data {
                None => {
                    match sub_msg.msg_responses.first() {
                        None => Err(StdError::generic_err("No submsg response")),
                        Some(response) => {
                            decode_and_handle_binary_data(deps, &response.value)
                        }
                    }
                }
                Some(bin) => {
                    decode_and_handle_binary_data(deps, &bin)
                }
            }
        }
    }
}