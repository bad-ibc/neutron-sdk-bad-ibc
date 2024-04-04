use thiserror::Error;

use cosmwasm_std::StdError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Bech32(#[from] bech32::Error),

    #[error("{0}")]
    Decode(#[from] prost::DecodeError),

    #[error("{0}")]
    DeSerde(#[from] serde_json_wasm::de::Error),
}

pub type ContractResult<T> = Result<T, ContractError>;
