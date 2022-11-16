use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Invalid Owner")]
    InvalidOwner {},

    #[error("Invalid Coin")]
    InvalidCoin {},

    #[error("User does not have coins from this cw20 to withdraw")]
    NoCw20ToWithdraw {},

}
