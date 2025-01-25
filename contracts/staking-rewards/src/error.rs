use cosmwasm_std::StdError;
use symphony_utils::denom::DenomError;
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

    #[error(transparent)]
    DenomError (#[from] DenomError),
}