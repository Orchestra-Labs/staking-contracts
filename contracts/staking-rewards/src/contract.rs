#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use std::collections::HashMap;

use crate::error::ContractError;
use crate::msg::{AllUserStatesResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, ListPoolStatesResponse, PoolStateResponse, QueryMsg, UserStateResponse};
use crate::state::{Config, PoolState, RewardsDistributionByToken, RewardsRecord, UserState, CONFIG, POOL_STATE, USER_STATE};
use cosmwasm_std::{to_json_binary, Addr, Binary, BlockInfo, DenomUnit, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult, Uint128, Uint64};
use cw2::set_contract_version;
use symphony_interfaces::staking::StakerBalanceResponse;

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
        ExecuteMsg::DistributeRewards => execute_distribute_rewards(deps, env, info),
        ExecuteMsg::ClaimRewards => execute_claim_rewards(deps, env, info),
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

fn execute_distribute_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = crate::state::CONFIG.load(deps.storage)?;

    let total_rewards_to_distribute = info.funds.iter().find(|coin| coin.denom == config.reward_token.denom)
        .map(|coin| coin.amount)
        .unwrap_or_default();

    if total_rewards_to_distribute.is_zero() {
        return Err(ContractError::NoRewardsToDistribute {});
    }

    for distro in &config.rewards_distribution {
        let denom_rewards = total_rewards_to_distribute
            .checked_mul(Uint128::from(distro.weight))?
            .checked_div(Uint128::from(WEIGHT_TOTAL))?;

        if denom_rewards.is_zero() {
            continue;
        }

        let pool_state = POOL_STATE.load(deps.storage, &distro.denom.denom)?;

        let total_rewards = pool_state.total_rewards
            .checked_add(denom_rewards)?;

        let updated_pool_state = PoolState {
            denom: distro.clone().denom,
            total_rewards,
            block_height: Uint64::from(env.block.height),
        };

        POOL_STATE.save(deps.storage, &distro.denom.denom, &updated_pool_state, env.block.height)?;

        let stakers = query_staking_contract_by_denom(
            &deps,
            &distro.denom.denom,
            config.staking_orchestrator_addr.to_string(),
        )?;

        for staker in stakers.stakers {
            let mut user_state = USER_STATE.load(
                deps.storage,
                &Addr::unchecked(&staker.address)
            ).unwrap_or(UserState {
                reward_debt: Uint128::zero(),
                last_claim_block_height: Uint64::zero(),
                rewards_data: HashMap::new(),
            });
            let user_rewards = staker.balance
                .checked_mul(denom_rewards)?
                .checked_div(stakers.total_staked)?;

            let values = user_state.rewards_data
                .entry(distro.denom.denom.clone())
                .or_insert_with(|| RewardsRecord { rewards: Uint128::zero() });
            values.rewards = values.rewards.checked_add(user_rewards)?;

            let updated_user_state = UserState {
                reward_debt: user_state.reward_debt.checked_add(user_rewards)?,
                last_claim_block_height: user_state.last_claim_block_height,
                rewards_data: user_state.rewards_data,
            };

            USER_STATE.save(
                deps.storage,
                &Addr::unchecked(&staker.address),
                &updated_user_state,
                env.block.height
            )?;
        }
    }

    Ok(Response::new()
        .add_attribute("action", "distribute_rewards")
        .add_attribute("total_rewards_to_distribute", total_rewards_to_distribute)
    )
}

const STAKERS_LIMIT: u32 = 1000u32;

struct StakingBag {
    stakers: Vec<StakerBalanceResponse>,
    total_staked: Uint128,
}

fn query_staking_contract_by_denom(
    deps: &DepsMut,
    denom: &str,
    orchestrator_addr: String
) -> Result<StakingBag, ContractError> {
    let mut start_after = None;
    let limit = Some(STAKERS_LIMIT);
    let mut stakers_acc: Vec<StakerBalanceResponse> = vec![];

    let query_request = &symphony_interfaces::orchestrator::QueryMsg::ListStakersByDenom {
        denom: denom.to_string(),
        start_after,
        limit,
    };
    let response: symphony_interfaces::orchestrator::ListStakersByDenomResponse = deps.querier
        .query_wasm_smart(orchestrator_addr.clone(), query_request)?;
    // add all response stakers to stakers_acc
    stakers_acc.extend(response.stakers);

    loop {
        let last_staker = stakers_acc.last();
        match last_staker {
            None => return Ok(StakingBag {
                stakers: vec![],
                total_staked: Uint128::zero(),
            }),
            Some(last_staker) => {
                start_after = Some(last_staker.address.clone());
                let query_request = &symphony_interfaces::orchestrator::QueryMsg::ListStakersByDenom {
                    denom: denom.to_string(),
                    start_after,
                    limit,
                };
                let response: symphony_interfaces::orchestrator::ListStakersByDenomResponse = deps.querier
                    .query_wasm_smart(orchestrator_addr.clone(), query_request)?;
                if response.stakers.is_empty() {
                    return Ok(StakingBag {
                        stakers: stakers_acc.clone(),
                        total_staked: stakers_acc
                            .clone()
                            .iter()
                            .fold(Uint128::zero(), |acc, x| acc + x.balance),
                    });
                }
                stakers_acc.extend(response.stakers);
            }
        }
    }
}

fn execute_claim_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let user_state = USER_STATE.load(deps.storage, &info.sender)?;

    // send rewards to user
    let total_rewards = user_state.reward_debt;
    let rewards_data = user_state.rewards_data;
    let rev_denom = config.reward_token.denom.clone();

    if total_rewards.is_zero() {
        return Err(ContractError::NoRewardsToClaim {});
    }

    let rewards_msg = cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![cosmwasm_std::Coin {
            denom: rev_denom,
            amount: total_rewards,
        }],
    });

    for reward in rewards_data {
        let denom = reward.0;
        let record = reward.1;
        let pool_state = POOL_STATE.load(deps.storage, &denom)?;

        let updated_total_rewards = pool_state.total_rewards.checked_sub(record.rewards)?;
        let updated_pool_state = PoolState {
            denom: pool_state.denom,
            total_rewards: updated_total_rewards,
            block_height: Uint64::from(env.block.height),
        };

        POOL_STATE.save(deps.storage, &denom, &updated_pool_state, env.block.height)?;
    }

    let updated_user_state = UserState {
        reward_debt: Uint128::zero(),
        last_claim_block_height: Uint64::from(env.block.height),
        rewards_data: HashMap::new(),
    };

    USER_STATE.save(deps.storage, &info.sender, &updated_user_state, env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "claim_rewards")
        .add_attribute("total_rewards", total_rewards)
        .add_message(rewards_msg)
    )
}

// fn query_contract_bank_balance(deps: &DepsMut, denom: &str, contract_addr: &str) -> Result<Uint128, ContractError> {
//     let balance_request = QueryRequest::Bank(BankQuery::Balance {
//         address: contract_addr.to_string(),
//         denom: denom.to_string(),
//     });
//
//     let balance_response: Coin = deps.querier.query(&balance_request)?;
//
//     let balance = balance_response.amount;
//
//     Ok(balance)
// }

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::AllPoolStates {} => to_json_binary(&query_all_pool_states(deps)?),
        QueryMsg::PoolState { denom, block_height } => to_json_binary(&query_pool_state(deps, denom, block_height)?),
        QueryMsg::AllUserStates {} => to_json_binary(&query_all_user_states(deps)?),
        QueryMsg::UserState { address, block_height } => to_json_binary(&query_user_state(deps, address, block_height)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        staking_orchestrator_addr: config.staking_orchestrator_addr.to_string(),
        reward_token: config.reward_token.clone(),
        rewards_distribution: config.rewards_distribution.clone(),
    })
}

fn query_all_pool_states(deps: Deps) -> StdResult<ListPoolStatesResponse> {
    let pool_states: Vec<PoolState> = POOL_STATE
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (_, pool_state) = item?;
            Ok(pool_state)
        })
        .collect::<StdResult<Vec<PoolState>>>()?;

    let states = pool_states.iter().map(|pool_state| {
        PoolStateResponse {
            denom: pool_state.denom.clone(),
            total_rewards: pool_state.total_rewards,
            block_height: pool_state.block_height,
        }
    }).collect();

    let pool_states = ListPoolStatesResponse {
        pool_states: states,
    };

    Ok(pool_states)
}

fn query_pool_state(deps: Deps, denom: String, block_height: Option<Uint64>) -> StdResult<PoolStateResponse> {
    let pool_state = match block_height {
        None => Some(POOL_STATE.load(deps.storage, &denom)?),
        Some(height) => POOL_STATE.may_load_at_height(deps.storage, &denom, height.u64())?,
    };

    match pool_state {
        None => Err(StdError::not_found(denom)),
        Some(pool_state) => Ok(PoolStateResponse {
            denom: pool_state.denom.clone(),
            total_rewards: pool_state.total_rewards,
            block_height: pool_state.block_height,
        }),
    }
}

fn query_all_user_states(
    deps: Deps,
) -> StdResult<AllUserStatesResponse> {
    let user_states: Vec<UserStateResponse> = USER_STATE
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (address, user_state) = item?;
            Ok(UserStateResponse {
                address: address.to_string(),
                reward_debt: user_state.reward_debt,
                rewards_data: user_state.rewards_data,
            })
        })
        .collect::<StdResult<Vec<UserStateResponse>>>()?;

    let user_states = AllUserStatesResponse {
        user_states,
    };

    Ok(user_states)
}

fn query_user_state(deps: Deps, address: String, block_height: Option<Uint64>) -> StdResult<UserStateResponse> {
    let user_state = match block_height {
        None => Some(USER_STATE.load(deps.storage, &Addr::unchecked(&address))?),
        Some(height) => USER_STATE.may_load_at_height(deps.storage, &Addr::unchecked(&address), height.u64())?,
    };

    match user_state {
        None => Err(StdError::not_found(address)),
        Some(user_state) => Ok(UserStateResponse {
            address,
            reward_debt: user_state.reward_debt,
            rewards_data: user_state.rewards_data,
        })
    }
}