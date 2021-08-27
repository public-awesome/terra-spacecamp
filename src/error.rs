use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl From<cw721_base::ContractError> for ContractError {
    fn from(err: cw721_base::ContractError) -> Self {
        match err {
            cw721_base::ContractError::Std(error) => ContractError::Std(error),
        }
    }
}
