use super::error::ContractError;
use super::msg::ExecuteMsg::{ClaimRewards, DistributeRewards};
use super::msg::{AllUserStatesResponse, ConfigResponse, InstantiateMsg, ListPoolStatesResponse, PoolStateResponse, QueryMsg, UserStateResponse};
use super::state::RewardsDistributionByToken;
use cosmwasm_std::{coin, Addr, BlockInfo, DenomUnit, Empty, Uint128, Uint64};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

const OWNER: &str = "owner";
const STAKERA: &str = "stakera";
// const STAKERB: &str = "stakerb";
const STAKE_DENOM: &str = "ustake";
const REWARD_DENOM: &str = "urev";

const TIME_BETWEEN_BLOCKS: u64 = 5;

fn mock_app() -> App {
    App::default()
}

fn mint_native(app: &mut App, recipient: &str, denom: &str, amount: u128) {
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: recipient.to_string(),
            amount: vec![coin(amount, denom)],
        },
    ))
        .unwrap();
}

fn next_block(app: &mut App) {
    app.set_block(BlockInfo {
        height: app.block_info().height + 1,
        time: app.block_info().time.plus_seconds(TIME_BETWEEN_BLOCKS),
        chain_id: app.block_info().chain_id,
    });
}

pub fn native_staking_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        native_staking::contract::execute,
        native_staking::contract::instantiate,
        native_staking::contract::query,
    );
    Box::new(contract)
}

pub fn staking_orchestrator_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        staking_orchestrator::contract::execute,
        staking_orchestrator::contract::instantiate,
        staking_orchestrator::contract::query,
    ).with_reply(staking_orchestrator::contract::reply);

    Box::new(contract)
}

pub fn staking_rewards_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        super::contract::execute,
        super::contract::instantiate,
        super::contract::query,
    );
    Box::new(contract)
}

fn instantiate_orchestrator(app: &mut App, denom: &str) -> Addr {
    let owner = app.api().addr_make(OWNER);
    let staking_code_id = app.store_code(native_staking_contract());
    let orchestrator_code_id = app.store_code(staking_orchestrator_contract());

    let msg = staking_orchestrator::msg::InstantiateMsg {
        owner: Some(owner.to_string()),
    };

    let orchestrator_addr = app.instantiate_contract(
        orchestrator_code_id,
        owner.clone(),
        &msg,
        &[],
        "orchestrator",
        Some(app.api().addr_make("admin").into()),
    ).unwrap();

    let execute_msg = staking_orchestrator::msg::ExecuteMsg::CreateStakingContract {
        code_id: staking_code_id,
        denom_unit: DenomUnit {
            denom: denom.to_string(),
            exponent: 6,
            aliases: vec![],
        },
        unbonding_period: None,
        owner: Some(owner.to_string())
    };

    app.execute_contract(
        owner,
        orchestrator_addr.clone(),
        &execute_msg,
        &[],
    ).unwrap();

    orchestrator_addr
}

fn instantiate_rewards(app: &mut App, owner: Option<String>, orchestrator_addr: &Addr, reward_denom: &DenomUnit, rewards_distribution: &Vec<RewardsDistributionByToken>) -> Addr {
    let rewards_code_id = app.store_code(staking_rewards_contract());
    let msg = InstantiateMsg {
        owner,
        staking_orchestrator_addr: orchestrator_addr.into(),
        reward_token: reward_denom.clone(),
        rewards_distribution: rewards_distribution.clone(),
    };
    app.instantiate_contract(
        rewards_code_id,
        app.api().addr_make(OWNER).into(),
        &msg,
        &[],
        "staking_rewards",
        Some(app.api().addr_make("admin").into()),
    )
        .unwrap()
}

#[test]
pub fn staking_rewards_instantiate() {
    let mut app = mock_app();
    let owner_address = app.api().addr_make(OWNER);
    mint_native(&mut app, &owner_address.to_string(), "urev", 1_000_000_000);
    let orchestrator_contract = app.api().addr_make("orchestrator");
    let reward_denom = DenomUnit {
        denom: "urev".to_string(),
        exponent: 6,
        aliases: vec![],
    };
    let rewards_distribution = vec![
        RewardsDistributionByToken {
            denom: DenomUnit {
                denom: "ustake".to_string(),
                exponent: 6,
                aliases: vec![],
            },
            weight: Uint64::from(50_000u64),
        },
        RewardsDistributionByToken {
            denom: DenomUnit {
                denom: "ustable".to_string(),
                exponent: 6,
                aliases: vec![],
            },
            weight: Uint64::from(50_000u64),
        }
    ];

    let rewards_contract = instantiate_rewards(
        &mut app,
        Some(owner_address.into()),
        &orchestrator_contract,
        &reward_denom,
        &rewards_distribution,
    );

    let _ = app.contract_data(&rewards_contract).unwrap();

    // check config
    let config: ConfigResponse = app.wrap().query_wasm_smart(
        rewards_contract.clone(),
        &QueryMsg::Config {},
    ).unwrap();

    assert_eq!(config.staking_orchestrator_addr, orchestrator_contract.into_string());
    assert_eq!(config.reward_token, reward_denom);
    assert_eq!(config.rewards_distribution, rewards_distribution);

    let pool_states: ListPoolStatesResponse = app.wrap().query_wasm_smart(
        rewards_contract.clone(),
        &QueryMsg::AllPoolStates {},
    ).unwrap();

    assert_eq!(pool_states.pool_states.len(), 2);
    println!("{:?}", pool_states);
}

fn stake_some_tokens(app: &mut App, user: &Addr, orchestrator_addr: &Addr, denom: &str, amount: u128) {
    mint_native(app, user.as_str(), denom, amount);


    let query = staking_orchestrator::msg::QueryMsg::StakingContractByDenom {
        denom: denom.to_string(),
    };
    let response: staking_orchestrator::msg::StakingContractByDenomResponse = app.wrap().query_wasm_smart(
        orchestrator_addr.clone(),
        &query,
    ).unwrap();
    let denom_staking_contract_addr = response.registered_contract.address;

    let msg = native_staking::msg::ExecuteMsg::Stake {};

    app.execute_contract(
        user.clone(),
        Addr::unchecked(denom_staking_contract_addr),
        &msg,
        &[coin(amount, denom)],
    ).unwrap();
}

#[test]
pub fn distribute_rewards() {
    let mut app = mock_app();
    let owner_address = app.api().addr_make(OWNER);
    let staker_a = app.api().addr_make(STAKERA);

    let orchestrator_addr = instantiate_orchestrator(&mut app, "ustake");

    let reward_denom = DenomUnit {
        denom: REWARD_DENOM.to_string(),
        exponent: 6,
        aliases: vec![],
    };
    let rewards_distribution = vec![
        RewardsDistributionByToken {
            denom: DenomUnit {
                denom: STAKE_DENOM.to_string(),
                exponent: 6,
                aliases: vec![],
            },
            weight: Uint64::from(100_000u64),
        },
    ];

    let rewards_contract = instantiate_rewards(
        &mut app,
        Some(owner_address.clone().to_string()),
        &orchestrator_addr,
        &reward_denom,
        &rewards_distribution,
    );

    let _ = app.contract_data(&rewards_contract).unwrap();

    stake_some_tokens(&mut app, &staker_a, &orchestrator_addr, STAKE_DENOM, 100);
    next_block(&mut app);

    mint_native(&mut app, owner_address.as_str(), REWARD_DENOM, 1_000_000);

    let msg = DistributeRewards {};
    let err = app.execute_contract(
        owner_address.clone(),
        rewards_contract.clone(),
        &msg,
        &[],
    ).unwrap_err();

    assert_eq!(err.root_cause().to_string(), ContractError::NoRewardsToDistribute {}.to_string());

    app.execute_contract(
        owner_address.clone(),
        rewards_contract.clone(),
        &msg,
        &[coin(1_000_000, REWARD_DENOM)],
    ).unwrap();

    let user_states: AllUserStatesResponse = app.wrap().query_wasm_smart(
        rewards_contract.clone(),
        &QueryMsg::AllUserStates {},
    ).unwrap();

    assert_eq!(user_states.user_states.len(), 1);
    assert_eq!(user_states.user_states[0].reward_debt, Uint128::from(1_000_000u128));

    let pool_state: PoolStateResponse = app.wrap().query_wasm_smart(
        rewards_contract.clone(),
        &QueryMsg::PoolState {
            denom: STAKE_DENOM.to_string(),
            block_height: None,
        },
    ).unwrap();

    assert_eq!(pool_state.total_rewards, Uint128::from(1_000_000u128));

    let user_state: UserStateResponse = app.wrap().query_wasm_smart(
        rewards_contract.clone(),
        &QueryMsg::UserState {
            address: staker_a.to_string(),
            block_height: None,
        },
    ).unwrap();

    println!("{:?}", user_state);
    assert_eq!(user_state.reward_debt, Uint128::from(1_000_000u128));
    next_block(&mut app);

    let msg = ClaimRewards {};
    app.execute_contract(
        staker_a.clone(),
        rewards_contract.clone(),
        &msg,
        &[],
    ).unwrap();

    let user_state: UserStateResponse = app.wrap().query_wasm_smart(
        rewards_contract.clone(),
        &QueryMsg::UserState {
            address: staker_a.to_string(),
            block_height: None,
        },
    ).unwrap();

    assert_eq!(user_state.reward_debt, Uint128::zero());

    // check staker_a balance
    let balance = app.wrap().query_balance(staker_a.clone(), REWARD_DENOM).unwrap();
    assert_eq!(balance.amount, Uint128::from(1_000_000u128));

    let pool_state: PoolStateResponse = app.wrap().query_wasm_smart(
        rewards_contract.clone(),
        &QueryMsg::PoolState {
            denom: STAKE_DENOM.to_string(),
            block_height: None,
        },
    ).unwrap();

    assert_eq!(pool_state.total_rewards, Uint128::zero());
}