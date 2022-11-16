use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Uint128, Addr, Coin};
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw20Deposits {
    pub count: i32,
    pub owner: String,
    pub contract:String,
    pub amount:Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Deposits {
    pub count: i32,
    pub owner: Addr,
    pub coins: Coin
}

//key is address, denom
pub const DEPOSITS: Map<(&str, &str), Deposits> = Map::new("deposits");

//key is address, cw20 contract address
pub const CW20_DEPOSITS: Map<(&str, &str), Cw20Deposits> = Map::new("cw20deposits");
