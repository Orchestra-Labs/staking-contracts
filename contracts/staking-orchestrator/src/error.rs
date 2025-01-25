use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error(transparent)]
    ParseReply(#[from] cw_utils::ParseReplyError),

    #[error("Reply message id is unknown")]
    UnknownReplyId {
        id: u64,
    },

    #[error("Cannot instantiate staking contract")]
    SubContractInstantiationFailed {},
}