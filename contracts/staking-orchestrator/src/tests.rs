use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{Addr, DenomUnit, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

const OWNER: &str = "owner";
const TIME_BETWEEN_BLOCKS: u64 = 5;

fn mock_app() -> App {
    App::default()
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

    let err = app.execute_contract(
        owner_address.clone(),
        orchestrator_contract.clone(),
        &msg,
        &[],
    ).unwrap();
}
