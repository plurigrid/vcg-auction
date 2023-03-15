use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Auction {
    pub id: u64,
    pub in_progress: bool,
    pub max_participants: Uint128,
    pub sorted_bids: Vec<Bid>,
    pub winning_bid: Option<Bid>,
}

#[cw_serde]
pub struct Bid {
    pub auction_id: u64,
    pub amount: Uint128,
    pub bidder: Addr,
    pub timestamp: Timestamp,
}

pub const AUCTIONS: Map<&u64, Auction> = Map::new("auctions");

pub const CURRENT_AUCTION_ID: Item<&u64> = Item::new("current_auction_id");

pub const BIDDERS_TO_BIDS: Map<&Addr, Vec<Uint128>> = Map::new("bidders_to_bids");

impl Auction {
    // O(1)
    pub fn new(id: u64, max_participants: Uint128) -> Self {
        Auction {
            id,
            in_progress: false,
            max_participants,
            sorted_bids: Vec::new(),
            winning_bid: None,
        }
    }

    // O(1)
    pub fn start(&mut self) {
        self.in_progress = true;
    }

    // O(1)
    pub fn end(&mut self) {
        self.in_progress = false;
    }

    // O(1)
    pub fn is_in_progress(&self) -> bool {
        self.in_progress
    }

    // O(log n) - average case
    // O(n) - worst case
    pub fn add_bid(&mut self, bid: Bid) -> Result<(), &str> {
        if self.sorted_bids.len() >= self.max_participants.u128() as usize {
            return Err("Maximum number of participants reached");
        }
        let index = match self
            .sorted_bids
            .binary_search_by_key(&bid.amount, |b| b.amount)
        {
            Ok(index) => index,
            Err(index) => index,
        };
        self.sorted_bids.insert(index, bid);
        Ok(())
    }

    // O(1) - average case
    // O(n) - worst case
    pub fn get_highest_bid(&self) -> Option<&Bid> {
        self.sorted_bids
            .iter()
            .rev()
            .max_by_key(|bid| (bid.amount, bid.timestamp))
    }

    // O(1) - average case
    // O(n) - worst case
    pub fn get_second_highest_bid(&self) -> Option<&Bid> {
        if self.sorted_bids.len() < 2 {
            None
        } else {
            let highest_bid = self.get_highest_bid().unwrap();
            self.sorted_bids
                .iter()
                .filter(|bid| bid != &highest_bid)
                .max_by_key(|bid| (bid.amount, bid.timestamp))
        }
    }
}
