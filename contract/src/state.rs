use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Storage, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};

use crate::ContractError;

#[cw_serde]
pub struct Auction {
    pub id: u64,
    pub name: String,
    pub in_progress: bool,
    pub max_participants: Uint64,
    pub sorted_bids: Vec<Bid>,
    pub winner: Option<Winner>,
}

#[cw_serde]
pub struct Bid {
    pub auction_id: u64,
    pub amount: Uint128,
    pub bidder: Addr,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub struct Winner {
    pub auction_id: u64,
    pub amount_owed: Uint128,
    pub bidder: Addr,
}

pub const AUCTIONS: Map<u64, Auction> = Map::new("auctions");

pub const CURRENT_AUCTION_ID: Item<u64> = Item::new("current_auction_id");

pub const BIDDERS_TO_BIDS: Map<(&Addr, u64), Bid> = Map::new("bidders_to_bids");

impl Auction {
    // O(1)
    pub fn new(id: u64, max_participants: Uint64, name: String) -> Self {
        Auction {
            id,
            in_progress: false,
            max_participants,
            sorted_bids: Vec::new(),
            winner: None,
            name,
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
    pub fn add_bid(&mut self, storage: &mut dyn Storage, bid: Bid) -> Result<(), ContractError> {
        if self.sorted_bids.len() >= self.max_participants.u64() as usize {
            return Err(ContractError::MaxParticipantsReached {
                max_participants: self.max_participants,
            });
        }
        let index = match self
            .sorted_bids
            .binary_search_by_key(&bid.amount, |b| b.amount)
        {
            Ok(index) => index,
            Err(index) => index,
        };
        self.sorted_bids.insert(index, bid);
        AUCTIONS.save(storage, self.id, &self)?;
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
