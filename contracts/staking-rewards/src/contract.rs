#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, PoolState, RewardsDistributionByToken, CONFIG, POOL_STATE};
use cosmwasm_std::{to_json_binary, BankQuery, Binary, BlockInfo, Coin, DenomUnit, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest, Response, StdResult, Uint128, Uint64};
use cw2::set_contract_version;

pub(crate) const CONTRACT_NAME: &str = "crates.io:symphony-staking-rewards";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const WEIGHT_TOTAL: u64 = 100_000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg.owner.unwrap_or(info.sender.to_string());
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;

    let orchestrator_addr = deps.api.addr_validate(&msg.staking_orchestrator_addr)?;
    validate_distribution(&msg.rewards_distribution)?;

    let config = Config {
        staking_orchestrator_addr: orchestrator_addr.clone(),
        reward_token: msg.reward_token.clone(),
        rewards_distribution: msg.rewards_distribution.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    let pool_states = init_pool_state(
        &msg.rewards_distribution.iter().map(|x| x.denom.clone()).collect::<Vec<DenomUnit>>(),
        &env.block,
    );

    for pool_state in pool_states {
        POOL_STATE.save(deps.storage, &pool_state.denom.denom, &pool_state, env.block.height)?;
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("staking_orchestrator_addr", orchestrator_addr)
        .add_attribute("reward_token", msg.reward_token.denom))
}

fn init_pool_state(denoms: &[DenomUnit], block_info: &BlockInfo) -> Vec<PoolState> {
    denoms.iter().map(|denom| PoolState {
        denom: denom.clone(),
        total_rewards: Uint128::zero(),
        total_staked: Uint128::zero(),
        block_height: Uint64::from(block_info.height),
    }).collect()
}

fn validate_distribution(distribution: &Vec<RewardsDistributionByToken>) -> Result<Uint64, ContractError> {
    let total_weight = distribution
        .iter()
        .fold(Uint64::zero(), |acc, x| acc + x.weight);
    if total_weight != Uint64::from(WEIGHT_TOTAL) {
        return Err(ContractError::InvalidRewardsDistribution {
            total_weight: total_weight.u64(),
        });
    }
    Ok(total_weight)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut,
               env: Env,
               info: MessageInfo,
               msg: ExecuteMsg) -> Result<Response<Empty>, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => execute_update_owner(deps, info, env, action),
        ExecuteMsg::UpdateConfig {
            staking_orchestrator_addr,
            reward_token,
            rewards_distribution,
        } => execute_update_config(deps, info, staking_orchestrator_addr, reward_token, rewards_distribution),
        ExecuteMsg::UpdateRewardsState => execute_update_rewards_state(deps, info),
    }
}

fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::default().add_attributes(ownership.into_attributes()))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    staking_orchestrator_addr: Option<String>,
    reward_token: Option<DenomUnit>,
    rewards_distribution: Option<Vec<RewardsDistributionByToken>>,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = crate::state::CONFIG.load(deps.storage)?;
    if let Some(staking_orchestrator_addr) = staking_orchestrator_addr {
        config.staking_orchestrator_addr = deps.api.addr_validate(&staking_orchestrator_addr)?;
    };

    if let Some(reward_token) = reward_token {
        config.reward_token = reward_token;
    };

    if let Some(rewards_distribution) = rewards_distribution {
        validate_distribution(&rewards_distribution)?;
        config.rewards_distribution = rewards_distribution;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("staking_orchestrator_addr", config.staking_orchestrator_addr)
        .add_attribute("reward_token", config.reward_token.denom)
    )
}

fn execute_update_rewards_state(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let balance = query_contract_bank_balance(&deps, &config.reward_token.denom, &info.sender.to_string())?;

    Ok(Response::new()
        .add_attribute("action", "update_rewards_state")
        .add_attribute("balance", balance)
    )
}

fn query_contract_bank_balance(deps: &DepsMut, denom: &str, contract_addr: &str) -> Result<Uint128, ContractError> {
    let balance_request = QueryRequest::Bank(BankQuery::Balance {
        address: contract_addr.to_string(),
        denom: denom.to_string(),
    });

    let balance_response: Coin = deps.querier.query(&balance_request)?;

    let balance = balance_response.amount;

    Ok(balance)
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}