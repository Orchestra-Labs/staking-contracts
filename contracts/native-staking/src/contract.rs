#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, ListStakersResponse, MigrateMsg, QueryMsg, TotalStakedAtHeightResponse};
use crate::state::{Config, CLAIMS, CONFIG, MAX_CLAIMS, STAKED_BALANCES, STAKED_TOTAL};
use cosmwasm_std::{coin, to_json_binary, Addr, BankMsg, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult, Uint128};
use cw2::set_contract_version;
use cw_controllers::ClaimsResponse;
use cw_ownable::get_ownership;
use cw_storage_plus::Bound;
use symphony_interfaces::staking::{InstantiateMsg, StakedBalanceAtHeightResponse, StakerBalanceResponse};
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
        staking_token: msg.denom_unit.clone(),
        unstaking_duration: msg.unbonding_period,
    };

    CONFIG.save(deps.storage, &config)?;

    STAKED_TOTAL.save(deps.storage, &Uint128::zero(), env.block.height)?;
    Ok(
        Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("owner", owner)
            .add_attribute("denom", msg.denom_unit.denom)
            .add_attribute("token_exponent", msg.denom_unit.exponent.to_string())
    )
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
        ExecuteMsg::Unstake { amount } => execute_unstake(deps, env, info, amount),
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
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

pub fn execute_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let staked_total = STAKED_TOTAL.load(deps.storage)?;
    let user_balance = STAKED_BALANCES.load(deps.storage, &info.sender);
    if user_balance.is_err() {
        return Err(ContractError::NoUnstakeAmount {});
    }

    if staked_total.is_zero() {
        return Err(ContractError::NoStakeAmount {});
    }

    if staked_total.saturating_add(amount) == Uint128::MAX {
        return Err(ContractError::AvoidSaturationAttack {});
    }

    if amount > staked_total {
        return Err(ContractError::InvalidUnstakeAmount {});
    }

    if amount > user_balance? {
        return Err(ContractError::InvalidUnstakeAmount {});
    }

    STAKED_BALANCES.update(
        deps.storage,
        &info.sender,
        env.block.height,
        |bal| -> StdResult<Uint128> { Ok(bal.unwrap_or_default().checked_sub(amount)?) },
    )?;
    STAKED_TOTAL.update(
        deps.storage,
        env.block.height,
        |total| -> StdResult<Uint128> {
            // Initialized during instantiate - OK to unwrap.
            Ok(total.unwrap().checked_sub(amount)?)
        },
    )?;

    match config.unstaking_duration {
        None => {
            // send the tokens back to the sender
            let contract_balance = deps.querier.query_balance(
                env.contract.address,
                config.staking_token.denom.clone()
            );

            match contract_balance {
                Ok(balance) => {
                    if balance.amount >= amount {
                        let msg: BankMsg = BankMsg::Send {
                            to_address: info.sender.to_string(),
                            amount: vec![
                                coin(amount.u128(), config.staking_token.denom.as_str())
                            ],
                        };
                        Ok(
                            Response::new()
                                .add_message(msg)
                                .add_attribute("action", "unstake")
                                .add_attribute("from", info.sender)
                                .add_attribute("denom", config.staking_token.denom)
                                .add_attribute("amount", amount)
                        )
                    } else {
                        Err(ContractError::InvalidUnstakeAmount {})
                    }
                },
                Err(_) => Err(ContractError::NoUnstakeAmount {})
            }
        }
        Some(duration) => {
            let pending_claims = CLAIMS.query_claims(deps.as_ref(), &info.sender)?.claims;
            if pending_claims.len() >= MAX_CLAIMS as usize {
                return Err(ContractError::TooManyClaims {});
            }

            CLAIMS.create_claim(deps.storage, &info.sender, amount, duration.after(&env.block))?;

            Ok(Response::new()
                .add_attribute("action", "unstake")
                .add_attribute("from", info.sender)
                .add_attribute("denom", config.staking_token.denom)
                .add_attribute("amount", amount)
                .add_attribute("claim_duration",format!("{duration}")))
        }
    }
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mature_claims = CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, None)?;
    if mature_claims.is_zero() {
        return Err(ContractError::NothingToClaim {})
    }
    let config = CONFIG.load(deps.storage)?;
    let msg: BankMsg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![coin(mature_claims.u128(), config.staking_token.denom.as_str())],
    };
    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "claim")
        .add_attribute("from", info.sender)
        .add_attribute("denom", config.staking_token.denom)
        .add_attribute("amount", mature_claims))
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
        QueryMsg::Claims { address} => to_json_binary(&query_claims(deps, address)?),
        QueryMsg::ListStakers { start_after, limit } => {
            to_json_binary(&query_all_stakers(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
}

pub fn query_staked_balance(deps: Deps, env: Env, address: String, height: Option<u64>) -> StdResult<StakedBalanceAtHeightResponse> {
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

pub fn query_claims(
    deps: Deps,
    address: String,
) -> StdResult<ClaimsResponse> {
    CLAIMS.query_claims(deps, &deps.api.addr_validate(&address)?)
}

pub fn query_all_stakers(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<ListStakersResponse> {
    let valid_start_addr: Option<Addr> = start_after
        .as_ref()
        .map(|addr| deps.api.addr_validate(addr))
        .transpose()?;

    let start_addr = valid_start_addr
        .as_ref()
        .map(|addr| Bound::exclusive(addr));

    let num_elements = match limit {
        Some(limit) => limit as usize,
        None => usize::MAX,
    };

    let stakers: Vec<StakerBalanceResponse> = STAKED_BALANCES
        .range(deps.storage, start_addr, None, cosmwasm_std::Order::Ascending)
        .take(num_elements)
        .filter_map(|item| item.ok()) // Gracefully handle potential errors.
        .map(|(addr, balance)| StakerBalanceResponse {
            address: addr.to_string(),
            balance,
        })
        .collect();

    Ok(ListStakersResponse { stakers })
}