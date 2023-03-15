use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint64};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ExecuteStartAuction {
        /// The name of the auction.
        name: String,
        /// The number of participants in the auction.
        /// Each participant may only bid once.
        number_of_participants: Uint64,
    },
    /// Allows a participant to bid in the auction.
    ExecuteBid {
        /// The participant's bid.
        bid_amount: Uint128,
    },
    /// Allows anyone to end the auction.
    /// The auction may only be ended once the total number of bids has been received.
    ExecuteCloseAuction {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cosmwasm_std::Addr)]
    QueryGetWinner { auction_id: u64 },
    #[returns(Vec<Uint128>)]
    QueryGetBidsForBidder {
        bidder: String,
        start_after: Option<Uint128>,
        limit: Option<u32>,
    },
    #[returns(Vec<Uint128>)]
    QueryGetBidsForAuction {
        auction_id: u64,
        start_after: Option<Uint128>,
        limit: Option<u32>,
    },
}
