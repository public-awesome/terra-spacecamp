use cosmwasm_std::Coin;
use cw721_base::msg::MintMsg as Cw721MintMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Ask, Bid};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    pub base: Cw721MintMsg,
    pub ask_amount: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint(MintMsg),
    SetBid {
        token_id: String,
        amount: Coin,
        bidder: String,
    },
    AcceptBid {
        token_id: String,
        bidder: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CurrentAskForToken { token_id: String },
    BidForTokenBidder { token_id: String, bidder: String },
    OwnerOf { token_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentAskForTokenResponse {
    pub ask: Ask,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidForTokenBidderResponse {
    pub bid: Bid,
}
