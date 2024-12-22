use cosmwasm_std::{coin, Addr, BlockInfo, Coin, DenomUnit, Empty, Uint128};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use cw_ownable::{Action, Ownership};
use cw_utils::Duration;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StakedBalanceAtHeightResponse, TotalStakedAtHeightResponse};
use crate::state::Config;

const OWNER: &str = "owner";
const TIME_BETWEEN_BLOCKS: u64 = 5;

fn mock_app() -> App {
    App::default()
}

fn mint_native(app: &mut App, recipient: String, denom: String, amount: u128) {
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: recipient,
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
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
        .with_migrate(crate::contract::migrate);
    Box::new(contract)
}

fn instantiate_staking(app: &mut App, owner: Option<String>, native_token: &DenomUnit, unbounding_duration: &Option<Duration>) -> Addr {
    let staking_code_id = app.store_code(native_staking_contract());
    let msg = InstantiateMsg {
        owner: owner,
        denom_unit: native_token.clone(),
        unbonding_period: unbounding_duration.clone(),
    };
    app.instantiate_contract(
        staking_code_id,
        app.api().addr_make(OWNER).into(),
        &msg,
        &[],
        "staking",
        Some(app.api().addr_make("admin").into()),
    )
        .unwrap()
}

#[test]
pub fn native_staking_instantiate() {
    let mut app = &mut mock_app();
    let native_token = DenomUnit {
        denom: "ustake".to_string(),
        exponent: 6,
        aliases: vec![],
    };
    let unbounding_duration = Some(Duration::Time(100));
    let owner_address = app.api().addr_make(OWNER);
    let staking_contract = instantiate_staking(
        &mut app,
        Some(owner_address.into()),
        &native_token,
        &unbounding_duration
    );

    let config: Config = app.wrap().query_wasm_smart(staking_contract.clone(), &QueryMsg::Config {}).unwrap();
    assert_eq!(config.staking_token, native_token);
    assert_eq!(config.unstaking_duration, unbounding_duration);
    let owner: Ownership<String> = app.wrap().query_wasm_smart(staking_contract, &QueryMsg::Ownership {}).unwrap();
    assert_eq!(owner.owner, Some(app.api().addr_make(OWNER).into()));
}

#[test]
pub fn native_staking_instantiate_without_explicit_owner() {
    let mut app = &mut mock_app();
    let native_token = DenomUnit {
        denom: "ustake".to_string(),
        exponent: 6,
        aliases: vec![],
    };
    let unbounding_duration = Some(Duration::Time(100));
    let staking_contract = instantiate_staking(
        &mut app,
        None,
        &native_token,
        &unbounding_duration
    );

    let config: Config = app.wrap().query_wasm_smart(staking_contract.clone(), &QueryMsg::Config {}).unwrap();
    assert_eq!(config.staking_token, native_token);
    assert_eq!(config.unstaking_duration, unbounding_duration);
    let owner: Ownership<String> = app.wrap().query_wasm_smart(staking_contract, &QueryMsg::Ownership {}).unwrap();
    assert_eq!(owner.owner, Some(app.api().addr_make(OWNER).into()));
}

#[test]
pub fn update_ownership() {
    let mut app = &mut mock_app();
    let sender = app.api().addr_make(OWNER);
    let native_token = DenomUnit {
        denom: "ustake".to_string(),
        exponent: 6,
        aliases: vec![],
    };
    let unbounding_duration = Some(Duration::Time(100));
    let staking_contract = instantiate_staking(
        &mut app,
        None,
        &native_token,
        &unbounding_duration
    );

    let new_owner = app.api().addr_make("new_owner");
    let msg = ExecuteMsg::UpdateOwnership(Action::TransferOwnership {
        new_owner: new_owner.clone().into(),
        expiry: None,
    });
    app.execute_contract(sender, staking_contract.clone(), &msg, &[]).unwrap();

    let owner: Ownership<String> = app.wrap().query_wasm_smart(staking_contract.clone(), &QueryMsg::Ownership {}).unwrap();
    assert_eq!(owner.pending_owner, Some(new_owner.clone().into()));

    let msg = ExecuteMsg::UpdateOwnership(Action::AcceptOwnership {});
    app.execute_contract(new_owner.clone(), staking_contract.clone(), &msg, &[]).unwrap();

    let owner: Ownership<String> = app.wrap().query_wasm_smart(staking_contract, &QueryMsg::Ownership {}).unwrap();
    assert_eq!(owner.owner, Some(new_owner.into()));
}

#[test]
pub fn execute_stake_should_fail_if_no_funds() {
    let mut app = &mut mock_app();
    let sender = app.api().addr_make(OWNER);
    let native_token = DenomUnit {
        denom: "ustake".to_string(),
        exponent: 6,
        aliases: vec![],
    };
    let unbounding_duration = Some(Duration::Time(100));
    let staking_contract = instantiate_staking(
        &mut app,
        None,
        &native_token,
        &unbounding_duration
    );

    let msg = ExecuteMsg::Stake {};
    let err = app.execute_contract(sender, staking_contract, &msg, &[]).unwrap_err();
    assert_eq!(err.root_cause().to_string(), ContractError::NoStakeAmount {}.to_string());
}

#[test]
pub fn execute_stake_should_fail_if_more_than_one_coin_sent() {
    let mut app = &mut mock_app();
    let sender = app.api().addr_make(OWNER);
    let native_token = DenomUnit {
        denom: "ustake".to_string(),
        exponent: 6,
        aliases: vec![],
    };
    let unbounding_duration = Some(Duration::Time(100));
    let staking_contract = instantiate_staking(
        &mut app,
        None,
        &native_token,
        &unbounding_duration
    );

    // add funds to sender address
    mint_native(&mut app, sender.to_string(), "ustake".to_string(), 100_000u128);
    mint_native(&mut app, sender.to_string(), "utoken".to_string(), 100_000u128);

    let msg = ExecuteMsg::Stake {};
    let err = app.execute_contract(sender, staking_contract, &msg, &[
        Coin {
            denom: "ustake".to_string(),
            amount: Uint128::zero(),
        },
        Coin {
            denom: "utoken".to_string(),
            amount: Uint128::from(100u128),
        }
    ]).unwrap_err();
    assert_eq!(err.root_cause().to_string(), ContractError::InvalidDenom {}.to_string());
}

#[test]
pub fn execute_stake_should_succeed() {
    let mut app = &mut mock_app();
    let sender = app.api().addr_make(OWNER);
    let native_token = DenomUnit {
        denom: "ustake".to_string(),
        exponent: 6,
        aliases: vec![],
    };
    let unbounding_duration = Some(Duration::Time(100));
    let staking_contract = instantiate_staking(
        &mut app,
        None,
        &native_token,
        &unbounding_duration
    );

    // add funds to sender address
    mint_native(&mut app, sender.to_string(), "ustake".to_string(), 100_000u128);

    let msg = ExecuteMsg::Stake {};
    app.execute_contract(sender.clone(), staking_contract.clone(), &msg, &[
        Coin {
            denom: "ustake".to_string(),
            amount: Uint128::from(100u128),
        }
    ]);

    next_block(&mut app);

    // query staked balance
    let staked_balance: StakedBalanceAtHeightResponse = app.wrap().query_wasm_smart(staking_contract.clone(), &QueryMsg::StakedBalanceAtHeight {
        address: sender.to_string(),
        height: None,
    }).unwrap();

    assert_eq!(staked_balance.balance, Uint128::from(100u128));

    // query total staked balance
    let total_staked_balance: TotalStakedAtHeightResponse = app.wrap().query_wasm_smart(staking_contract, &QueryMsg::TotalStakedAtHeight {
        height: None,
    }).unwrap();

    assert_eq!(total_staked_balance.total, Uint128::from(100u128));
}