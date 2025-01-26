use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::{Config, RewardsDistributionByToken};
use cosmwasm_std::{coin, Addr, BlockInfo, DenomUnit, Empty, Uint64};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

const OWNER: &str = "owner";
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

pub fn staking_rewards_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
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
    let config: Config = app.wrap().query_wasm_smart(
        rewards_contract.clone(),
        &QueryMsg::Config {},
    ).unwrap();

    assert_eq!(config.staking_orchestrator_addr, orchestrator_contract);
    assert_eq!(config.reward_token, reward_denom);
    assert_eq!(config.rewards_distribution, rewards_distribution);
}