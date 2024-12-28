use crate::msg::{AllTokensStakedBalanceAtHeightResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StakingContractByDenomResponse};
use cosmwasm_std::{coin, Addr, BlockInfo, Coin, DenomUnit, Empty, Uint128};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

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

pub fn native_staking_contract() -> Box<dyn Contract<Empty>>{
    let contract = ContractWrapper::new(
        native_staking::contract::execute,
        native_staking::contract::instantiate,
        native_staking::contract::query,
    );
    Box::new(contract)
}

pub fn staking_orchestrator_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
      crate::contract::execute,
      crate::contract::instantiate,
      crate::contract::query,
    ).with_reply(crate::contract::reply);
    Box::new(contract)
}

fn instantiate_orchestrator(app: &mut App, owner: Option<String>) -> Addr {
    let orchestrator_code_id = app.store_code(staking_orchestrator_contract());
    let msg = InstantiateMsg { owner };
    app.instantiate_contract(
        orchestrator_code_id,
        app.api().addr_make(OWNER).into(),
        &msg,
        &[],
        "orchestrator",
        Some(app.api().addr_make("admin").into()),
    )
        .unwrap()
}

fn stake_tokens(app: &mut App, amount: Uint128, denom: &str, staking_contract: Addr) {
    let msg = native_staking::msg::ExecuteMsg::Stake {};

    app.execute_contract(
        app.api().addr_make(OWNER),
        staking_contract,
        &msg,
        &[Coin{
            denom: denom.to_string(),
            amount,
        }],
    ).unwrap();

}

#[test]
pub fn staking_orchestrator_instantiate() {
    let mut app = mock_app();
    let owner_address = app.api().addr_make(OWNER);
    let orchestator_contract = instantiate_orchestrator(
        &mut app,
        Some(owner_address.into()),
    );

    let _ = app.contract_data(&orchestator_contract).unwrap();
}

#[test]
pub fn execute_create_staking_contract() {
    let mut app = mock_app();
    let owner_address = app.api().addr_make(OWNER);
    let orchestrator_contract = instantiate_orchestrator(
        &mut app,
        Some(owner_address.clone().into()),
    );

    let staking_code_id = app.store_code(native_staking_contract());
    let denom_unit = "ustake";
    let msg = ExecuteMsg::CreateStakingContract {
        code_id: staking_code_id,
        denom_unit: DenomUnit {
            denom: denom_unit.to_string(),
            exponent: 6,
            aliases: vec![],
        },
        unbonding_period: None,
        owner: None,
    };

    let _ = app.execute_contract(
        owner_address.clone(),
        orchestrator_contract.clone(),
        &msg,
        &[],
    ).unwrap();

    let contract_data: StakingContractByDenomResponse = app.wrap().query_wasm_smart(
        orchestrator_contract.clone(),
        &QueryMsg::StakingContractByDenom {
            denom: denom_unit.to_string(),
        },
    ).unwrap();

    assert_eq!(contract_data.denom, denom_unit);
    assert_eq!(contract_data.registered_contract.token.denom, denom_unit);
}

#[test]
pub fn query_all_staked_tokens() {
    let mut app = mock_app();
    let owner_address = app.api().addr_make(OWNER);
    let orchestrator_contract = instantiate_orchestrator(
        &mut app,
        Some(owner_address.clone().into()),
    );

    let staking_code_id = app.store_code(native_staking_contract());
    let denom_unit = "ustake";
    let msg = ExecuteMsg::CreateStakingContract {
        code_id: staking_code_id,
        denom_unit: DenomUnit {
            denom: denom_unit.to_string(),
            exponent: 6,
            aliases: vec![],
        },
        unbonding_period: None,
        owner: None,
    };

    let _ = app.execute_contract(
        owner_address.clone(),
        orchestrator_contract.clone(),
        &msg,
        &[],
    ).unwrap();

    let denom_unit = "ucoin";
    let msg = ExecuteMsg::CreateStakingContract {
        code_id: staking_code_id,
        denom_unit: DenomUnit {
            denom: denom_unit.to_string(),
            exponent: 6,
            aliases: vec![],
        },
        unbonding_period: None,
        owner: None,
    };

    let _ = app.execute_contract(
        owner_address.clone(),
        orchestrator_contract.clone(),
        &msg,
        &[],
    ).unwrap();

    let all_staked: AllTokensStakedBalanceAtHeightResponse = app.wrap().query_wasm_smart(
        orchestrator_contract.clone(),
        &QueryMsg::AllTokensStakedBalanceAtHeight {
            address: owner_address.clone().into_string(),
            height: None
        }
    ).unwrap();

    assert_eq!(all_staked.tokens_staked_balance.len(), 2);
    assert_eq!(all_staked.tokens_staked_balance.get("ustake").unwrap().balance, Uint128::zero());
    assert_eq!(all_staked.tokens_staked_balance.get("ucoin").unwrap().balance, Uint128::zero());

    let contract_data: StakingContractByDenomResponse = app.wrap().query_wasm_smart(
        orchestrator_contract.clone(),
        &QueryMsg::StakingContractByDenom {
            denom: "ustake".to_string(),
        },
    ).unwrap();

    mint_native(&mut app, owner_address.clone().into_string(), "ustake".to_string(), 200);
    let staking_contract = contract_data.registered_contract.address;

    stake_tokens(
        &mut app,
        Uint128::new(100),
        "ustake",
        Addr::unchecked(staking_contract)
    );

    next_block(&mut app);

    let all_staked: AllTokensStakedBalanceAtHeightResponse = app.wrap().query_wasm_smart(
        orchestrator_contract.clone(),
        &QueryMsg::AllTokensStakedBalanceAtHeight {
            address: owner_address.clone().into_string(),
            height: None
        }
    ).unwrap();

    assert_eq!(all_staked.tokens_staked_balance.len(), 2);
    assert_eq!(all_staked.tokens_staked_balance.get("ustake").unwrap().balance, Uint128::new(100));
    assert_eq!(all_staked.tokens_staked_balance.get("ucoin").unwrap().balance, Uint128::zero());
}
