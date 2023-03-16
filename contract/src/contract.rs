#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage,
    Uint128, Uint64,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryAuctionWinnerResponse, QueryBidsForBidderResponse,
    QueryCurrentAuctionIdResponse, QueryMsg,
};
use crate::state::{Auction, Bid, Winner, AUCTIONS, BIDDERS_TO_BIDS, CURRENT_AUCTION_ID};

// version info for migration
const CONTRACT_NAME: &str = "crates.io:vcg-auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
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
            max_num_participants: number_of_participants,
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

    auction.add_bid(deps.storage, bid.clone())?;
    BIDDERS_TO_BIDS.save(deps.storage, (&info.sender, auction.id), &bid)?;

    Ok(Response::default())
}

fn execute_start_auction(
    deps: DepsMut,
    name: String,
    max_participants: Uint64,
) -> Result<Response, ContractError> {
    let auction_id = get_and_increment_auction_id(deps.storage)?;

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
        QueryMsg::QueryGetAuctionWinner { auction_id } => {
            return query_get_winner(deps, auction_id.u64())
        }
        QueryMsg::QueryGetBidsForBidder {
            bidder,
            start_after,
            limit,
        } => query_get_bids_for_bidder(deps, bidder, start_after, limit),
        QueryMsg::QueryGetBidsForAuction {
            auction_id,
            start_after,
            limit,
        } => query_get_bids_for_auction(deps, auction_id.u64(), start_after, limit),
        QueryMsg::QueryGetCurrentAuctionId {} => query_get_current_auction_id(deps),
    }
}

// TODO: paginate query
fn query_get_bids_for_bidder(
    deps: Deps,
    bidder: String,
    _start_after: Option<Uint128>,
    _limit: Option<u32>,
) -> StdResult<Binary> {
    let bidder = deps.api.addr_validate(&bidder)?;
    let bids = BIDDERS_TO_BIDS
        .prefix(&bidder)
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|pair| Ok(pair?.1))
        .collect::<StdResult<Vec<_>>>()?;

    return Ok(to_binary(&QueryBidsForBidderResponse { bids })?);
}

// TODO: paginate query
fn query_get_bids_for_auction(
    deps: Deps,
    auction_id: u64,
    _start_after: Option<Uint128>,
    _limit: Option<u32>,
) -> StdResult<Binary> {
    let auction: Auction = AUCTIONS
        .load(deps.storage, auction_id)
        .map_err(|_| StdError::generic_err(format!("auction with id {} not found", auction_id)))?;

    return Ok(to_binary(&auction.sorted_bids)?);
}

fn query_get_winner(deps: Deps, auction_id: u64) -> StdResult<Binary> {
    let auction: Auction = AUCTIONS
        .load(deps.storage, auction_id)
        .map_err(|_| StdError::generic_err("auction not found"))?;

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
            return Ok(to_binary(&QueryAuctionWinnerResponse { winner: winner })?);
        }
    }
}

fn query_get_current_auction_id(deps: Deps) -> StdResult<Binary> {
    let auction_id = CURRENT_AUCTION_ID.load(deps.storage)?;
    return Ok(to_binary(&QueryCurrentAuctionIdResponse {
        auction_id: auction_id.into(),
    })?);
}

pub fn get_and_increment_auction_id(storage: &mut dyn Storage) -> StdResult<u64> {
    let new_id = CURRENT_AUCTION_ID.update(storage, |id| -> StdResult<_> {
        let new_id = id + 1;
        Ok(new_id)
    })?;
    Ok(new_id)
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fmt::format;

    use crate::msg::{ExecuteMsg, QueryBidsForBidderResponse};
    use crate::state::{Bid, Winner};
    use cosmwasm_std::{Addr, Empty, StdError, Timestamp, Uint128, Uint64};
    use cw_multi_test::{App, Contract, ContractWrapper};
    use cw_multi_test::{AppResponse, Executor};
    use rand::Rng;

    fn auction_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(super::execute, super::instantiate, super::query);
        Box::new(contract)
    }

    fn instantiate_auction(app: &mut App) -> Addr {
        let code_id: u64 = app.store_code(auction_contract());
        let auction = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(ADMIN),
                &crate::msg::InstantiateMsg {},
                &[],
                "coin",
                None,
            )
            .unwrap();
        return auction;
    }

    // Returns current auction id
    fn start_auction(app: &mut App, auction: Addr) -> u64 {
        // Get current auction ID
        let resp = app
            .wrap()
            .query_wasm_smart::<crate::msg::QueryCurrentAuctionIdResponse>(
                auction.clone(),
                &crate::msg::QueryMsg::QueryGetCurrentAuctionId {},
            )
            .unwrap();

        let prev_auction_id = resp.auction_id.u64();

        // Start auction
        app.execute_contract(
            Addr::unchecked(ADMIN),
            auction.clone(),
            &ExecuteMsg::ExecuteStartAuction {
                max_num_participants: Uint64::from(10000u64),
                name: "auction_1".to_string(),
            },
            &[],
        )
        .unwrap();

        // Ensure current auction ID is correct
        let resp = app
            .wrap()
            .query_wasm_smart::<crate::msg::QueryCurrentAuctionIdResponse>(
                auction.clone(),
                &crate::msg::QueryMsg::QueryGetCurrentAuctionId {},
            )
            .unwrap();

        let current_auction_id = resp.auction_id.u64();
        assert_eq!(current_auction_id, prev_auction_id + 1);
        return current_auction_id;
    }

    fn close_auction(app: &mut App, auction: Addr) {
        app.execute_contract(
            Addr::unchecked(ADMIN),
            auction,
            &ExecuteMsg::ExecuteCloseAuction {},
            &[],
        )
        .unwrap();
    }

    fn bid(
        app: &mut App,
        auction: Addr,
        bidder: Addr,
        bid_amount: Uint128,
    ) -> Result<AppResponse, anyhow::Error> {
        return app.execute_contract(bidder, auction, &ExecuteMsg::ExecuteBid { bid_amount }, &[]);
    }

    const ADMIN: &str = "admin";

    #[test]
    fn test_auction_start_and_close() {
        let mut app = App::default();
        let auction = instantiate_auction(&mut app);
        start_auction(&mut app, auction.clone());
        close_auction(&mut app, auction);
    }

    #[test]
    fn test_query_bids_for_bidder() {
        let mut app = App::default();
        let auction = instantiate_auction(&mut app);
        let first_auction_id = start_auction(&mut app, auction.clone());

        let bidder1 = Addr::unchecked("bidder1");

        // Place bids
        bid(
            &mut app,
            auction.clone(),
            bidder1.clone(),
            Uint128::from(10u128),
        )
        .unwrap();

        let err = bid(
            &mut app,
            auction.clone(),
            bidder1.clone(),
            Uint128::from(20u128),
        )
        .unwrap_err();

        // assert can only bid once
        let err_str = format!("{:?}", err);
        assert!(err_str.contains("Participant has already placed a bid for this auction"));

        let bids: Vec<Bid> = app
            .wrap()
            .query_wasm_smart::<QueryBidsForBidderResponse>(
                auction.clone(),
                &crate::msg::QueryMsg::QueryGetBidsForBidder {
                    bidder: bidder1.clone().into(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
            .bids;

        assert_eq!(bids.len(), 1);
        assert_eq!(bids[0].auction_id, first_auction_id);
        assert_eq!(bids[0].bidder, bidder1);

        close_auction(&mut app, auction.clone());

        let new_auction_id = start_auction(&mut app, auction.clone());

        // bid again
        bid(
            &mut app,
            auction.clone(),
            bidder1.clone(),
            Uint128::from(20u128),
        )
        .unwrap();

        let bids: Vec<Bid> = app
            .wrap()
            .query_wasm_smart::<QueryBidsForBidderResponse>(
                auction.clone(),
                &crate::msg::QueryMsg::QueryGetBidsForBidder {
                    bidder: bidder1.clone().into(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
            .bids;

        assert_eq!(bids.len(), 2);

        assert_eq!(bids[0].auction_id, first_auction_id);
        assert_eq!(bids[0].bidder, bidder1);

        assert_eq!(bids[1].auction_id, new_auction_id);
        assert_eq!(bids[1].bidder, bidder1);

        // for bid in bids.iter() {
        //     assert_eq!(bid.auction_id, auction_id);
        //     assert_eq!(bid.bidder, bidder1);
        // }
    }

    #[test]
    fn test_auction_with_one_bid() {
        let mut app = App::default();
        let code_id = app.store_code(auction_contract());
        let auction = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(ADMIN),
                &crate::msg::InstantiateMsg {},
                &[],
                "coin",
                None,
            )
            .unwrap();

        let auction_id = start_auction(&mut app, auction.clone());

        // Place bid
        bid(
            &mut app,
            auction.clone(),
            Addr::unchecked("bidder1"),
            Uint128::from(100u128),
        );

        // Close auction
        close_auction(&mut app, auction.clone());

        let winner_res = app
            .wrap()
            .query_wasm_smart::<crate::msg::QueryMsg>(
                auction.clone(),
                &crate::msg::QueryMsg::QueryGetAuctionWinner {
                    auction_id: auction_id.into(),
                },
            )
            .unwrap_err();

        assert_eq!(
            winner_res,
            StdError::generic_err("Querier contract error: Generic error: Auction has only one bid, cannot determine winner")
        );
    }

    #[test]
    fn test_find_winner() {
        let mut app = App::default();
        let code_id = app.store_code(auction_contract());
        let auction = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(ADMIN),
                &crate::msg::InstantiateMsg {},
                &[],
                "coin",
                None,
            )
            .unwrap();

        let auction_id = start_auction(&mut app, auction.clone());

        let mut bids: Vec<Bid> = Vec::new();
        let num_bids = rand::thread_rng().gen_range(2..1000);
        for i in 1..=num_bids {
            let bidder_name = format!("bidder{}", i);
            let bid_amount = Uint128::from(u128::from(
                u32::try_from(rand::thread_rng().gen_range(1..1000)).unwrap(),
            ));
            bid(
                &mut app,
                auction.clone(),
                Addr::unchecked(&bidder_name),
                bid_amount,
            );
            bids.push(Bid {
                auction_id,
                amount: bid_amount,
                bidder: Addr::unchecked(&bidder_name),
                timestamp: Timestamp::from_seconds(0),
            });
        }

        // Query bids for auction
        let queried_bids: Vec<Bid> = app
            .wrap()
            .query_wasm_smart(
                auction.clone(),
                &crate::msg::QueryMsg::QueryGetBidsForAuction {
                    auction_id: auction_id.into(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();

        assert_eq!(queried_bids.len(), bids.len());

        // Close auction
        app.execute_contract(
            Addr::unchecked(ADMIN),
            auction.clone(),
            &ExecuteMsg::ExecuteCloseAuction {},
            &[],
        )
        .unwrap();

        // Query winner for auction
        let winner: Winner = app
            .wrap()
            .query_wasm_smart::<crate::msg::QueryAuctionWinnerResponse>(
                auction,
                &crate::msg::QueryMsg::QueryGetAuctionWinner {
                    auction_id: auction_id.into(),
                },
            )
            .unwrap()
            .winner;

        let winning_bid = bids.iter().max_by(|a, b| a.amount.cmp(&b.amount)).unwrap();
        assert_eq!(winner.bidder, winning_bid.bidder);
    }
}

/*
TODO:
// test generate only one bid and fail
// verify other properties of bid
// test tied bids
// query winner before auction is closed
// test bidding beyond max participants
 */
