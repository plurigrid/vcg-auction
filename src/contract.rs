#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
    Uint128, Uint64,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Auction, Bid, Winner, AUCTIONS, BIDDERS_TO_BIDS, CURRENT_AUCTION_ID};

// version info for migration
const CONTRACT_NAME: &str = "crates.io:vcg-auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Init
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CURRENT_AUCTION_ID.save(deps.storage, &0)?;

    Ok(Response::default())
}

// Handle all execute messages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ExecuteStartAuction {
            name,
            number_of_participants,
        } => execute_start_auction(deps, name, number_of_participants),
        ExecuteMsg::ExecuteBid { bid_amount } => execute_bid(deps, env, info, bid_amount),
        ExecuteMsg::ExecuteCloseAuction {} => execute_close_auction(deps),
    }
}

fn execute_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bid_amount: Uint128,
) -> Result<Response, ContractError> {
    let current_auction_id = CURRENT_AUCTION_ID.load(deps.storage)?;
    let mut auction: Auction = AUCTIONS
        .load(deps.storage, current_auction_id)
        .map_err(|_| ContractError::AuctionNotFound {
            auction_id: current_auction_id,
        })?;

    if !auction.in_progress {
        return Err(ContractError::AuctionNotInProgress {});
    }

    if BIDDERS_TO_BIDS
        .may_load(deps.storage, (&info.sender, current_auction_id))?
        .is_some()
    {
        return Err(ContractError::BidAlreadyPlaced {});
    }

    let bid = Bid {
        auction_id: current_auction_id,
        amount: bid_amount,
        bidder: info.sender.clone(),
        timestamp: env.block.time,
    };

    auction.add_bid(deps.storage, bid)?;
    BIDDERS_TO_BIDS.save(deps.storage, (&info.sender, auction.id), &bid)?;

    Ok(Response::default())
}

fn execute_start_auction(
    deps: DepsMut,
    name: String,
    max_participants: Uint64,
) -> Result<Response, ContractError> {
    let auction_id = CURRENT_AUCTION_ID.load(deps.storage)?;

    // Check if auction already exists
    if AUCTIONS.may_load(deps.storage, auction_id)?.is_some() {
        return Err(ContractError::AuctionAlreadyInProgress {});
    }

    // Create a new auction
    let auction = Auction {
        id: auction_id,
        in_progress: true,
        sorted_bids: vec![],
        name: name,
        max_participants,
        winner: None,
    };

    AUCTIONS.save(deps.storage, auction_id, &auction)?;

    Ok(Response::default())
}

fn execute_close_auction(deps: DepsMut) -> Result<Response, ContractError> {
    let auction_id = CURRENT_AUCTION_ID.load(deps.storage)?;
    AUCTIONS.update(
        deps.storage,
        auction_id,
        |auction: Option<Auction>| match auction {
            Some(mut auction) => {
                if !auction.in_progress {
                    return Err(ContractError::AuctionNotInProgress {});
                }
                auction.in_progress = false;
                Ok(auction)
            }
            None => Err(ContractError::AuctionNotFound { auction_id }),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryGetWinner { auction_id } => return query_get_winner(deps, auction_id),
        QueryMsg::QueryGetBidsForBidder {
            bidder,
            start_after,
            limit,
        } => query_get_bids_for_bidder(deps, bidder, start_after, limit),
        QueryMsg::QueryGetBidsForAuction {
            auction_id,
            start_after,
            limit,
        } => query_get_bids_for_auction(deps, auction_id, start_after, limit),
    }
}

// TODO: paginate query
fn query_get_bids_for_bidder(
    deps: Deps,
    bidder: String,
    start_after: Option<Uint128>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let bidder = deps.api.addr_validate(&bidder)?;
    let bids = BIDDERS_TO_BIDS
        .prefix_range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    return Ok(to_binary(&bids)?);
}

// TODO: paginate query
fn query_get_bids_for_auction(
    deps: Deps,
    auction_id: u64,
    start_after: Option<Uint128>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let auction: Auction = AUCTIONS
        .load(deps.storage, auction_id)
        .map_err(|e| StdError::generic_err(format!("auction with id {} not found", auction_id)))?;

    return Ok(to_binary(&auction.sorted_bids)?);
}

fn query_get_winner(deps: Deps, auction_id: u64) -> StdResult<Binary> {
    let auction: Auction = AUCTIONS
        .load(deps.storage, auction_id)
        .map_err(|e| StdError::generic_err("auction not found"))?;

    if auction.is_in_progress() {
        return Err(StdError::generic_err("Auction in progress"));
    }

    match auction.winner {
        Some(winner) => {
            return Ok(to_binary(&winner)?);
        }
        None => {
            let highest_bid = auction.get_highest_bid().ok_or(StdError::generic_err(
                "Auction has no bids, cannot determine winner",
            ))?;
            let second_highest_bid =
                auction
                    .get_second_highest_bid()
                    .ok_or(StdError::generic_err(
                        "Auction has only one bid, cannot determine winner",
                    ))?;
            let winner = Winner {
                bidder: highest_bid.bidder.clone(),
                auction_id,
                amount_owed: second_highest_bid.amount,
            };
            return Ok(to_binary(&winner)?);
        }
    }
}

// pub fn paginate_vector(
//     storage: &dyn Storage,
//     vector: Vec<T>,
//     start_after: Option<T>,
//     limit: Option<u32>,
//     order: Order,
// ) -> StdResult<Vec<T>> {
//     let (range_min, range_max) = match order {
//         Order::Ascending => (start_after.map(Bound::exclusive), None),
//         Order::Descending => (None, start_after.map(Bound::exclusive)),
//     };

//     let items = vector.range(storage, range_min, range_max, order);
//     match limit {
//         Some(limit) => Ok(items
//             .take(limit.try_into().unwrap())
//             .collect::<StdResult<_>>()?),
//         None => Ok(items.collect::<StdResult<_>>()?),
//     }
// }

pub fn increment_auction_id(deps: DepsMut) -> StdResult<()> {
    CURRENT_AUCTION_ID.update(deps.storage, |id| -> StdResult<_> {
        let new_id = id + 1;
        Ok(new_id)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {}
