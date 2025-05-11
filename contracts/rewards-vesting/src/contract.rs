#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, VestingConfiguration, VestingRecord};
use crate::vesting::VestingInstantiateMsg;
use cosmwasm_std::{to_json_binary, Coin, DepsMut, Empty, Env, MessageInfo, Response, SubMsg, WasmMsg};
use cw2::set_contract_version;

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
        ExecuteMsg::BatchCancel { .. } => {}
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
) -> Result<Response<Empty>, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    
    let mut sub_msgs: Vec<SubMsg> = vec![];
    for record in records.iter() {
        deps.api.addr_validate(&record.address)?;

        let sub_msg = instantiate_cw_vesting_contract(env.clone(), info.clone(), config.clone(), record);
        sub_msgs.push(sub_msg);
    }

    Ok(Response::default().add_submessages(sub_msgs))
}

fn instantiate_cw_vesting_contract(env: Env, info: MessageInfo, config: VestingConfiguration, record: &VestingRecord) -> SubMsg {
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
        denom: config.denom.clone(),
        schedule: config.schedule,
        start_time: config.start_time,
        vesting_duration_seconds: config.duration,
        unbonding_duration_seconds: 0,
    };

    let init_msg = WasmMsg::Instantiate {
        admin: Some(owner.clone()),
        code_id: config.contract_code_id,
        msg: to_json_binary(&msg.clone())?,
        funds: info.funds,
        label: format!("{} rewards vesting contract for user: {}", config.denom, record.address.clone()),
    };

    SubMsg::reply_on_success(init_msg, INSTANTIATE_VESTING_REPLY_ID)
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//
// }