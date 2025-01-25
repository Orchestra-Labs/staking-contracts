use cosmwasm_std::{Addr, QuerierWrapper};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DenomError {
    #[error("Invalid unstaking duration, unstaking duration cannot be 0")]
    InvalidDenom { denom: String},
}

pub fn validate_denom_exists(querier: &QuerierWrapper, denom: &str) -> Result<(), DenomError> {
    let fake_address = Addr::unchecked("fake_address");

    let balance = querier.balance(&fake_address, denom);

    if balance.is_err() {
        return Err(DenomError::InvalidDenom { denom: denom.to_string() });
    }

    Ok(())
}