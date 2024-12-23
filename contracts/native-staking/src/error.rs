use cosmwasm_std::StdError;
use symphony_utils::duration::UnboundingDurationError;
use thiserror::Error;

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

    #[error("No stake amount to unstake")]
    NoUnstakeAmount {},

    #[error("Avoid saturation attack")]
    AvoidSaturationAttack {},

    #[error("Invalid amount to unstake")]
    InvalidUnstakeAmount {},

    #[error("Too many pending claims")]
    TooManyClaims {},

    #[error("Nothing to claim")]
    NothingToClaim {},

    #[error(transparent)]
    Ownership(#[from] cw_ownable::OwnershipError),
}