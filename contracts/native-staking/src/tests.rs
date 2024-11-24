use cosmwasm_std::{Addr, DenomUnit, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use cw_utils::Duration;
use crate::msg::InstantiateMsg;

const OWNER: &str = "owner";

fn mock_app() -> App {
    App::default()
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

fn instantiate_staking(app: &mut App, native_token: DenomUnit, unbounding_duration: Option<Duration>) -> Addr {
    let staking_code_id = app.store_code(native_staking_contract());
    let msg = InstantiateMsg {
        owner: Some(OWNER.to_string()),
        denom_unit: native_token,
        unbonding_period: unbounding_duration,
    };
    app.instantiate_contract(
        staking_code_id,
        Addr::unchecked(OWNER),
        &msg,
        &[],
        "staking",
        Some("admin".to_string()),
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
    let staking_contract = instantiate_staking(&mut app, native_token, unbounding_duration);
    assert_eq!(staking_contract, Addr::unchecked("staking"));
}