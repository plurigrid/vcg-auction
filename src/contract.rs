#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
    Timestamp, Uint128,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Winner, AUCTION_IN_PROGRESS, BIDS_TO_BIDDERS, NUMBER_OF_PARTICIPANTS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vcg-auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    return Ok(Response::default());
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ExecuteStartAuction {
            number_of_participants,
        } => {
            return execute_start_auction(deps, number_of_participants);
        }
        ExecuteMsg::ExecuteBid { bid_amount } => {
            return execute_bid(deps, info, bid_amount);
        }
        ExecuteMsg::ExecuteCloseAuction {} => {
            return execute_close_auction(deps);
        }
    }
}

fn execute_bid(
    deps: DepsMut,
    info: MessageInfo,
    bid_amount: Uint128,
) -> Result<Response, ContractError> {
    // Check auction is started
    if !AUCTION_IN_PROGRESS.may_load(deps.storage)?.unwrap_or(false) {
        return Err(ContractError::AuctionAlreadyStarted {});
    }

    // Check bid amount is valid
    if bid_amount < Uint128::from(1u128) {
        return Err(ContractError::AuctionAlreadyStarted {});
    }

    // Check participant has not already bid
    if BIDS_TO_BIDDERS
        .may_load(deps.storage, bid_amount.u128())?
        .is_some()
    {
        return Err(ContractError::ParticipantAlreadyBid {});
    }

    // Add bid to map
    BIDS_TO_BIDDERS.save(deps.storage, bid_amount.u128(), &info.sender)?;
    return Ok(Response::default());
}

fn execute_start_auction(
    deps: DepsMut,
    number_of_participants: Uint128,
) -> Result<Response, ContractError> {
    // Check auction is not already started
    if AUCTION_IN_PROGRESS.may_load(deps.storage)?.unwrap_or(false) {
        return Err(ContractError::AuctionAlreadyStarted {});
    }

    // Check number of participants is at least 2
    if number_of_participants < Uint128::from(2u128) {
        return Err(ContractError::TooFewParticipants {});
    }

    // Save number of participants
    if !AUCTION_IN_PROGRESS.may_load(deps.storage)?.unwrap_or(false) {
        return Err(ContractError::AuctionNotStarted {});
    }

    // Set auction in progress to true
    AUCTION_IN_PROGRESS.save(deps.storage, &true)?;

    return Ok(Response::default());
}

fn execute_close_auction(deps: DepsMut) -> Result<Response, ContractError> {
    // Check auction is started
    if !AUCTION_IN_PROGRESS.may_load(deps.storage)?.unwrap_or(false) {
        return Err(ContractError::AuctionNotStarted {});
    }

    // Check number of bids is equal to number of participants
    if BIDS_TO_BIDDERS
        .keys(deps.storage, None, None, Order::Ascending)
        .count()
        != NUMBER_OF_PARTICIPANTS.load(deps.storage)?.u128() as usize
    {
        return Err(ContractError::AuctionNotStarted {});
    }

    // Set auction in progress to false
    AUCTION_IN_PROGRESS.save(deps.storage, &false)?;

    return Ok(Response::default());
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryFindWinner {} => {
            return query_find_winner(_deps);
        }
    }
}

fn query_find_winner(deps: Deps) -> StdResult<Binary> {
    // Check auction is closed
    if AUCTION_IN_PROGRESS.may_load(deps.storage)?.unwrap_or(false) {
        return Err(StdError::GenericErr {
            msg: "auction not closed".to_string(),
        });
    }

    // We find the lowest bid and return the address of the participant who made that bid, as well as the
    // amount of the second-lowest bid, which is what the winner will pay.

    let mut keys_iter = BIDS_TO_BIDDERS.keys(deps.storage, None, None, Order::Ascending);
    let lowest_bid: u128 = keys_iter
        .next()
        .transpose()?
        .ok_or(StdError::generic_err("no bids found"))?;

    let lowest_bid_winner = BIDS_TO_BIDDERS.load(deps.storage, lowest_bid)?;
    let second_lowest_bid = keys_iter
        .next()
        .transpose()?
        .ok_or(StdError::generic_err("no bids found"))?;

    let winner: Winner = Winner {
        bidder: lowest_bid_winner,
        amount_to_pay: second_lowest_bid,
    };

    return Ok(to_binary(&winner)?);
}

#[cfg(test)]
mod tests {}
