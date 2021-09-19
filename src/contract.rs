#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw721_base::state::tokens;

use crate::error::ContractError;
use crate::msg::{
    BidForTokenBidderResponse, CurrentAskForTokenResponse, ExecuteMsg, MintMsg, QueryMsg,
};
use crate::state::{Ask, Bid, TOKEN_ASKS, TOKEN_BIDDERS};
use cw721_base::contract::{
    _transfer_nft as cw721_transfer_nft, execute_mint as cw721_execute_mint,
    instantiate as cw721_instantiate,
};
use cw721_base::msg::InstantiateMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    cw721_instantiate(deps, _env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint(msg) => execute_mint(deps, env, info, msg),
        ExecuteMsg::SetBid {
            token_id,
            amount,
            bidder,
        } => execute_set_bid(deps, env, info, token_id, amount, bidder),
        ExecuteMsg::AcceptBid { token_id, bidder } => {
            execute_accept_bid(deps, env, info, token_id, bidder)
        }
    }
}

pub fn execute_mint(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    cw721_execute_mint(deps.branch(), env, info, msg.base.clone())?;

    let ask = Ask {
        amount: msg.ask_amount,
    };
    TOKEN_ASKS.save(deps.storage, &msg.base.token_id, &ask)?;

    Ok(Response::default())
}

pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    amount: Coin,
    bidder: String,
) -> Result<Response, ContractError> {
    let bidder_addr = deps.api.addr_validate(&bidder)?;
    if amount.amount.is_zero() {
        return Err(ContractError::InvalidBidAmount {});
    }
    println!("info: {:?} | token_id: {:?} | amount: {:?} | bidder: {:?}",info, token_id, amount, bidder);
    // save bid
    let bid = Bid {
        amount,
        bidder: bidder_addr.clone(),
    };
    TOKEN_BIDDERS.save(deps.storage, (&token_id, &bidder_addr), &bid)?;

    // check ask
    let ask = TOKEN_ASKS.load(deps.storage, &token_id)?;
    // make sure that bid's amount > ask's amount AND bid's denoms == ask's denoms
    if (bid.amount.amount > ask.amount.amount) && (bid.amount.denom == ask.amount.denom) {
        // transfer NFT
        transfer_nft(deps, env, info, bidder, token_id)?;
    }

    Ok(Response::new()
        .add_attribute("action", "set_bid"))
}

pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    bidder: String,
) -> Result<Response, ContractError> {
    // TODO: make sure alice is the owner
    let bidder_addr = deps.api.addr_validate(&bidder)?;
    let bid = TOKEN_BIDDERS.load(deps.storage, (&token_id, &bidder_addr))?;
    if bid.amount.amount.is_zero() {
        return Err(ContractError::InvalidBidAmount {});
    }

    // transfer NFT
    transfer_nft(deps, env, info, bidder, token_id)?;

    Ok(Response::new().add_attribute("action", "accept_bid"))
}

fn transfer_nft(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<(), ContractError> {
    // transfer NFT
    cw721_transfer_nft(deps.branch(), &env, &info, &recipient, &token_id)?;

    // remove the accepted bid
    let recipient_addr = deps.api.addr_validate(&recipient)?;
    TOKEN_BIDDERS.remove(deps.storage, (&token_id, &recipient_addr));
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CurrentAskForToken { token_id } => {
            to_binary(&query_current_ask_for_token(deps, token_id)?)
        }
        QueryMsg::BidForTokenBidder { token_id, bidder } => {
            to_binary(&query_bid_for_token_bidder(deps, token_id, bidder)?)
        }
        QueryMsg::OwnerOf { token_id } => to_binary(&query_owner_of(deps, token_id)?),
    }
}

fn query_current_ask_for_token(
    deps: Deps,
    token_id: String,
) -> StdResult<CurrentAskForTokenResponse> {
    let ask = TOKEN_ASKS.load(deps.storage, &token_id)?;
    Ok(CurrentAskForTokenResponse { ask })
}

fn query_bid_for_token_bidder(
    deps: Deps,
    token_id: String,
    bidder: String,
) -> StdResult<BidForTokenBidderResponse> {
    let bidder_addr = deps.api.addr_validate(&bidder)?;
    let bid = TOKEN_BIDDERS.load(deps.storage, (&token_id, &bidder_addr))?;
    Ok(BidForTokenBidderResponse { bid })
}

fn query_owner_of(deps: Deps, token_id: String) -> StdResult<String> {
    let info = tokens().load(deps.storage, &token_id)?;
    Ok(info.owner.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, Uint128};
    use cw721_base::msg::MintMsg as Cw721MintMsg;
    use cw721_base::state::num_tokens;

    const TOKEN_ID: &str = "123";
    const MINTER: &str = "minter_address";
    const ALICE: &str = "alice_address";
    const BOB: &str = "bob_address";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            name: "Cosmic Apes".into(),
            symbol: "APE".into(),
            minter: MINTER.into(),
        };
        let info = mock_info(MINTER, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(deps.as_mut());
    }

    #[test]
    fn mint() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(deps.as_mut());

        let mint_msg = ExecuteMsg::Mint(MintMsg {
            ask_amount: coin(5, "token"),
            base: Cw721MintMsg {
                token_id: TOKEN_ID.into(),
                owner: ALICE.into(),
                name: "Cosmic Ape 123".into(),
                description: Some("The first Cosmisc Ape".into()),
                image: None,
            },
        });

        let info = mock_info(MINTER, &[]);
        let _ = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // ensure num tokens increases
        let count = num_tokens(&deps.storage).unwrap();
        assert_eq!(1, count);

        // ensure ask is set
        let res = query_current_ask_for_token(deps.as_ref(), TOKEN_ID.into()).unwrap();
        assert_eq!(Uint128::from(5u128), res.ask.amount.amount);
    }

    #[test]
    fn set_bid() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(deps.as_mut());

        let mint_msg = ExecuteMsg::Mint(MintMsg {
            ask_amount: coin(5, "token"),
            base: Cw721MintMsg {
                token_id: TOKEN_ID.into(),
                owner: "owner".into(),
                name: "Cosmic Ape 123".into(),
                description: Some("The first Cosmisc Ape".into()),
                image: None,
            },
        });

        let info = mock_info(MINTER, &[]);
        let _ = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // bob bids 3
        let bid_msg = ExecuteMsg::SetBid {
            token_id: TOKEN_ID.into(),
            amount: coin(3, "token"),
            bidder: BOB.into(),
        };
        let bob_info = mock_info(BOB.into(), &[]);
        let _ = execute(deps.as_mut(), mock_env(), bob_info, bid_msg).unwrap();

        // ensure bob's bid was created (passes test but on-chain storage isn't working)
        let res = query_bid_for_token_bidder(deps.as_ref(), TOKEN_ID.into(), BOB.into()).unwrap();
        assert_eq!(Uint128::from(3u128), res.bid.amount.amount);
    }

    #[test]
    fn accept_bid() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(deps.as_mut());

        let mint_msg = ExecuteMsg::Mint(MintMsg {
            ask_amount: coin(5, "token"),
            base: Cw721MintMsg {
                token_id: TOKEN_ID.into(),
                owner: ALICE.into(),
                name: "Cosmic Ape 123".into(),
                description: Some("The first Cosmisc Ape".into()),
                image: None,
            },
        });

        let minter_info = mock_info(MINTER, &[]);
        let _ = execute(deps.as_mut(), mock_env(), minter_info, mint_msg).unwrap();

        // bob bids 3
        let bid_msg = ExecuteMsg::SetBid {
            token_id: TOKEN_ID.into(),
            amount: coin(3, "token"),
            bidder: BOB.into(),
        };
        let bob_info = mock_info(BOB.into(), &[]);
        let _ = execute(deps.as_mut(), mock_env(), bob_info, bid_msg).unwrap();

        // check if alice is the current owner of the NFT
        let owner = query_owner_of(deps.as_ref(), TOKEN_ID.into()).unwrap();
        assert_eq!(ALICE, owner);

        // alice accepts bob's bid
        let accept_bid_msg = ExecuteMsg::AcceptBid {
            token_id: TOKEN_ID.into(),
            bidder: BOB.into(),
        };
        let alice_info = mock_info(ALICE.into(), &[]);
        let _ = execute(deps.as_mut(), mock_env(), alice_info, accept_bid_msg).unwrap();

        // check if bob is the new owner of the NFT
        let owner = query_owner_of(deps.as_ref(), TOKEN_ID.into()).unwrap();
        assert_eq!(BOB, owner);
    }
}
