use cosmwasm_std::StdError;
use thiserror::Error;
use symphony_utils::duration::UnboundingDurationError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    UnboundingDurationError(#[from] UnboundingDurationError),

    #[error("No stake amount")]
    NoStakeAmount {},

    #[error("Invalid denom to stake")]
    InvalidDenom {},

    #[error(transparent)]
    Ownership(#[from] cw_ownable::OwnershipError),
}