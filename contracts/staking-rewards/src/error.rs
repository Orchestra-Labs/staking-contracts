use cosmwasm_std::{DivideByZeroError, OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error("Rewards distribution weights do not sum up to 100")]
    InvalidRewardsDistribution {
        total_weight: u64,
    },

    #[error("Invalid rewards amount, should be greater than zero")]
    NoRewardsToDistribute,

    #[error(transparent)]
    OverflowError(#[from] OverflowError),

    #[error(transparent)]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("There are no rewards to claim")]
    NoRewardsToClaim,

    #[error("Cannot execute this action while the contract is paused")]
    ContractPaused,

}