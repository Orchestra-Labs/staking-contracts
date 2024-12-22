#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StakedBalanceAtHeightResponse, TotalStakedAtHeightResponse};
use crate::state::{Config, CONFIG, STAKED_BALANCES, STAKED_TOTAL};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult, Uint128};
use cw2::set_contract_version;
use cw_ownable::get_ownership;
use symphony_utils::duration::validate_duration;

pub(crate) const CONTRACT_NAME: &str = "crates.io:symphony-native-staking";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<Empty>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner = msg.owner.as_deref().unwrap_or(info.sender.as_str());
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner))?;

    validate_duration(msg.unbonding_period)?;

    let config= Config {
        staking_token: msg.denom_unit,
        unstaking_duration: msg.unbonding_period,
    };

    CONFIG.save(deps.storage, &config)?;

    STAKED_TOTAL.save(deps.storage, &Uint128::zero(), env.block.height)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut,
               env: Env,
               info: MessageInfo,
               msg: ExecuteMsg) -> Result<Response<Empty>, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => execute_update_owner(deps, info, env, action),
        ExecuteMsg::UpdateConfig { unbonding_period } => execute_update_config(deps, env, info, unbonding_period),
        ExecuteMsg::Stake {} => execute_stake(deps, env, info),
    }
}

pub fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::default().add_attributes(ownership.into_attributes()))
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    unbonding_period: Option<cw_utils::Duration>,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    validate_duration(unbonding_period)?;

    CONFIG.update(deps.storage, |mut config| -> Result<Config, StdError> {
        config.unstaking_duration = unbonding_period;
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute(
            "unstaking_duration",
            unbonding_period
                .map(|d| format!("{d}"))
                .unwrap_or_else(|| "none".to_string()),
        ))
}

pub fn execute_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sender = info.sender;
    deps.api.addr_validate(&sender.as_str())?;

    if info.funds.is_empty() {
        return Err(ContractError::NoStakeAmount {});
    }

    if info.funds.iter().any(|c| c.denom != config.staking_token.denom) {
        return Err(ContractError::InvalidDenom {});
    }

    let amount_to_stake = info.funds.iter().find(|c| c.denom == config.staking_token.denom)
        .map(|c| c.amount).unwrap_or_default();

    if amount_to_stake.is_zero() {
        return Err(ContractError::NoStakeAmount {});
    }

    STAKED_BALANCES.update(
        deps.storage,
        &sender,
        env.block.height,
        |bal| -> StdResult<Uint128> { Ok(bal.unwrap_or_default().checked_add(amount_to_stake)?) },
    )?;
    STAKED_TOTAL.update(
        deps.storage,
        env.block.height,
        |total| -> StdResult<Uint128> {
            // Initialized during instantiate - OK to unwrap.
            Ok(total.unwrap().checked_add(amount_to_stake)?)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "stake")
        .add_attribute("from", sender)
        .add_attribute("denom", config.staking_token.denom)
        .add_attribute("amount", amount_to_stake))
}

//TODO: Implement migration logic
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response<Empty>, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Ownership {} => to_json_binary(&get_ownership(deps.storage)?),
        QueryMsg::StakedBalanceAtHeight { address, height } => to_json_binary(&query_staked_balance(deps, env, address, height)?),
        QueryMsg::TotalStakedAtHeight { height } => to_json_binary(&query_total_staked_at_height(deps, env, height)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
}

fn query_staked_balance(deps: Deps, env: Env, address: String, height: Option<u64>) -> StdResult<StakedBalanceAtHeightResponse> {
    let query_address = deps.api.addr_validate(&address)?;
    let query_height = height.unwrap_or(env.block.height);
    let balance = STAKED_BALANCES.may_load_at_height(deps.storage, &query_address, query_height)?.unwrap_or_default();
    Ok(StakedBalanceAtHeightResponse { balance, height: query_height })
}

pub fn query_total_staked_at_height(
    deps: Deps,
    _env: Env,
    height: Option<u64>,
) -> StdResult<TotalStakedAtHeightResponse> {
    let height = height.unwrap_or(_env.block.height);
    let total = STAKED_TOTAL
        .may_load_at_height(deps.storage, height)?
        .unwrap_or_default();
    Ok(TotalStakedAtHeightResponse { total, height })
}