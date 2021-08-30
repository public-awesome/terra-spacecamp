use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidBidAmount")]
    InvalidBidAmount {},

    #[error("Claimed")]
    Claimed {},

    #[error("Expired")]
    Expired {},
}

impl From<cw721_base::ContractError> for ContractError {
    fn from(err: cw721_base::ContractError) -> Self {
        match err {
            cw721_base::ContractError::Std(error) => ContractError::Std(error),
            cw721_base::ContractError::Unauthorized {} => ContractError::Unauthorized {},
            cw721_base::ContractError::Claimed {} => ContractError::Claimed {},
            cw721_base::ContractError::Expired {} => ContractError::Expired {},
        }
    }
}
