#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, RewardsDistributionByToken, CONFIG};
use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response, Uint64};
use cw2::set_contract_version;
use symphony_utils::denom::validate_denom_exists;

pub(crate) const CONTRACT_NAME: &str = "crates.io:symphony-staking-rewards";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    let orchestrator_addr = deps.api.addr_validate(&msg.staking_orchestrator_addr)?;
    validate_distribution(&msg.rewards_distribution)?;
    validate_denom_exists(&deps.querier, &msg.reward_token.denom)?;

    let config = Config {
        staking_orchestrator_addr: orchestrator_addr.clone(),
        reward_token: msg.reward_token.clone(),
        rewards_distribution: msg.rewards_distribution.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("staking_orchestrator_addr", orchestrator_addr)
        .add_attribute("reward_token", msg.reward_token.denom))
}

fn validate_distribution(distribution: &Vec<RewardsDistributionByToken>) -> Result<Uint64, ContractError> {
    let total_weight = distribution
        .iter()
        .fold(Uint64::zero(), |acc, x| acc.checked_add(x.weight)?)?;
    if total_weight != Uint64::from(100u64) {
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
        // ExecuteMsg::UpdateConfig {
        //     staking_orchestrator_addr,
        //     reward_token,
        //     rewards_distribution,
        // } => execute_update_config(deps, info, staking_orchestrator_addr, reward_token, rewards_distribution),
        // ExecuteMsg::Distribute {} => execute_distribute(deps, env),
        // ExecuteMsg::Withdraw {} => execute_withdraw(deps, env, info),
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

