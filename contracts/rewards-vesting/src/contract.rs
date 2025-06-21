#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VestingConfiguration, VestingContractResponse, VestingRecord};
use crate::state::VESTING_CONTRACTS;
use crate::vesting::{query_vesting_contract_for_info, UncheckedDenom, VestingExecuteMsg, VestingInstantiateMsg};
use cosmwasm_std::{to_json_binary, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, WasmMsg};
use cw2::set_contract_version;
use cw_utils::parse_instantiate_response_data;

pub(crate) const CONTRACT_NAME: &str = "crates.io:symphony-rewards-vesting";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_VESTING_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
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
        ExecuteMsg::BatchVesting {
            configuration,
            records,
        } => execute_batch_vesting(deps, env, info, configuration, records),
        ExecuteMsg::BatchCancel { addresses } => execute_batch_cancel(deps, env, info, addresses),
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

fn execute_batch_vesting(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: VestingConfiguration,
    records: Vec<VestingRecord>,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    
    let mut sub_msgs: Vec<SubMsg> = vec![];
    for record in records.iter() {
        deps.api.addr_validate(&record.address)?;

        let sub_msg = instantiate_cw_vesting_contract(env.clone(), info.clone(), config.clone(), record);
        sub_msgs.push(sub_msg?);
    }

    Ok(Response::default()
        .add_attribute("action", "create_batch_vesting")
        .add_attribute("denom", config.denom.clone())
        .add_attribute("owner", env.contract.address.into_string())
        .add_attribute("code_id", config.contract_code_id.to_string())
        .add_submessages(sub_msgs))
}

fn instantiate_cw_vesting_contract(env: Env, _info: MessageInfo, config: VestingConfiguration, record: &VestingRecord) -> StdResult<SubMsg> {
    let owner = env.contract.address.into_string();
    let funds = vec![Coin {
        denom: config.denom.clone(),
        amount: record.amount,
    }];
    let msg = VestingInstantiateMsg {
        owner: Some(owner.clone()),
        recipient: record.address.clone(),
        title: format!("{} {}", config.title_prefix, record.address),
        description: Some(format!("{} {}", config.description_prefix, record.address)),
        total: record.amount,
        denom: UncheckedDenom::Native(config.denom.clone()),
        schedule: config.schedule,
        start_time: config.start_time,
        vesting_duration_seconds: config.duration,
        unbonding_duration_seconds: 0,
    };

    let init_msg = WasmMsg::Instantiate {
        admin: Some(owner.clone()),
        code_id: config.contract_code_id,
        msg: to_json_binary(&msg.clone())?,
        funds,
        label: format!("{} rewards vesting contract for user: {}", config.denom, record.address.clone()),
    };

    Ok(SubMsg::reply_on_success(init_msg, INSTANTIATE_VESTING_REPLY_ID))
}

fn execute_batch_cancel(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    
    let contract_addresses = addresses.iter()
        .map(|addr| VESTING_CONTRACTS.load(deps.storage, addr.to_string()))
        .collect::<StdResult<Vec<String>>>()?;
    
   let cancel_msgs_result: Result<Vec<SubMsg>, ContractError> = contract_addresses.iter()
            .map(|address| {
                let msg = VestingExecuteMsg::Cancel {};
                let msg_binary = to_json_binary(&msg)
                    .map_err(ContractError::Std)?;
                let cancel_msg = WasmMsg::Execute {
                    contract_addr: address.clone(),
                    msg: msg_binary,
                    funds: vec![],
                };
                Ok(SubMsg::new(cancel_msg))
            })
            .collect();
        
        let cancel_msgs = cancel_msgs_result?;

    Ok(Response::default()
        .add_attribute("action", "batch_cancel_vesting")
        .add_attribute("addresses", addresses.join(", "))
        .add_submessages(cancel_msgs)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VestingContract { address } =>to_json_binary(&query_vesting_contract(deps, address)?),
    }
}

fn query_vesting_contract(
    deps: Deps,
    address: String,
) -> StdResult<VestingContractResponse> {
    let vesting_contract = VESTING_CONTRACTS.may_load(deps.storage, address.clone())?;

    Ok(VestingContractResponse {
        found: vesting_contract.is_some(),
        address: address.clone(),
        contract_address: vesting_contract,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        INSTANTIATE_VESTING_REPLY_ID => {
            handle_instantiate_vesting_contract_reply(deps, env, msg)
        },
        _ => Err(StdError::generic_err("Invalid reply ID")),
    }
}

fn decode_and_handle_binary_data(deps: DepsMut, bin: &Binary) -> StdResult<Response> {
    let decoded = parse_instantiate_response_data(&bin)
        .map_err(|e| StdError::generic_err(format!("parsing submsg response: {}", e)))?;
    
    let contract_info = query_vesting_contract_for_info(
        deps.as_ref(),
        decoded.contract_address.clone()
    )?;

    let recipient = contract_info.recipient.clone().into_string();

    VESTING_CONTRACTS.save(deps.storage, recipient, &decoded.contract_address)?;

    Ok(
        Response::new()
            .add_attribute("action", "create_vesting_contract")
            .add_attribute("denom", contract_info.denom.to_string())
            .add_attribute("recipient", contract_info.recipient.into_string())
            .add_attribute("title", contract_info.title)
            .add_attribute("description", contract_info.description.unwrap_or_default())
            .add_attribute("address", decoded.contract_address)
    )
}

#[allow(deprecated)]
fn handle_instantiate_vesting_contract_reply(
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