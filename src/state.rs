use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    pub amount: Coin,
    pub bidder: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    pub amount: Coin,
}

/// mapping from (token, bidder) -> bid
pub const TOKEN_BIDDERS: Map<(&str, &Addr), Bid> = Map::new("token_bidders");

/// mapping from token -> current ask for the token
pub const TOKEN_ASKS: Map<&str, Ask> = Map::new("token asks");
