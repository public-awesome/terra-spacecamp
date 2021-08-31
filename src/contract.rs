#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::helpers::Cw721Contract;
use crate::msg::{
    BidForTokenBidderResponse, CurrentAskForTokenResponse, ExecuteMsg, MintMsg, QueryMsg,
};
use crate::state::{Ask, Bid, TOKEN_ASKS, TOKEN_BIDDERS};
use cw721_base::contract::{execute_mint as cw721_execute_mint, instantiate as cw721_instantiate};
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
    _info: MessageInfo,
    token_id: String,
    amount: Coin,
    bidder: String,
) -> Result<Response, ContractError> {
    let bidder_addr = deps.api.addr_validate(&bidder)?;
    if amount.amount.is_zero() {
        return Err(ContractError::InvalidBidAmount {});
    }

    // send funds from bidder to contract
    let msg = BankMsg::Send {
        to_address: env.contract.address.into(),
        amount: vec![amount.clone()],
    };

    // save bid
    let bid = Bid {
        amount,
        bidder: bidder_addr.clone(),
    };
    TOKEN_BIDDERS.save(deps.storage, (&token_id, &bidder_addr), &bid)?;

    // check ask
    let ask = TOKEN_ASKS.load(deps.storage, &token_id)?;
    if bid.amount.amount > ask.amount.amount {
        // finalize transfer NFT
        todo!();
    }

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "set_bid"))
}

pub fn execute_accept_bid(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    token_id: String,
    bidder: String,
) -> Result<Response, ContractError> {
    let bidder_addr = deps.api.addr_validate(&bidder)?;
    let bid = TOKEN_BIDDERS.load(deps.storage, (&token_id, &bidder_addr))?;
    if bid.amount.amount.is_zero() {
        return Err(ContractError::InvalidBidAmount {});
    }

    // finalize NFT transfer
    todo!();

    //     Ok(Response::default())
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, QuerierWrapper, Uint128};
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
        let info = mock_info("creator", &[]);
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
                owner: "owner".into(),
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

        // ensure bob's bid was created
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
                owner: "owner".into(),
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

        // alice accepts bob's bid
        let accept_bid_msg = ExecuteMsg::AcceptBid {
            token_id: TOKEN_ID.into(),
            bidder: BOB.into(),
        };
        let alice_info = mock_info(ALICE.into(), &[]);
        let _ = execute(deps.as_mut(), mock_env(), alice_info, accept_bid_msg).unwrap();

        // TODO: check if bob is the new owner of the NFT
        // let owner_of_query = Cw721QueryMsg::OwnerOf {
        //     token_id: TOKEN_ID.into(),
        //     include_expired: Some(true),
        // };
        // let res = cw721_query(deps.as_ref(), mock_env(), owner_of_query).unwrap();
        // assert_eq!(BOB.into(), res.owner);
        let query_wrapper = QuerierWrapper::new(&deps.querier);
        let cw721 = Cw721Contract(mock_env().contract.address);
        let res = cw721.owner_of(&query_wrapper, TOKEN_ID, true).unwrap();
        assert_eq!(BOB, res.owner);
    }
}
