use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const AUCTION_IN_PROGRESS: Item<bool> = Item::new("auction_in_progress");

pub const NUMBER_OF_PARTICIPANTS: Item<Uint128> = Item::new("number_of_participants");

// We map bids amounts to bidders so that we can find the lowest bid in <= O(logn) time (depending on the implementation of prefixes in Cosmwasm, need to check)
pub const BIDS_TO_BIDDERS: Map<u128, Addr> = Map::new("bids_to_bidders");

pub const BIDDERS_TO_BIDS: Map<&Addr, Uint128> = Map::new("bidders_to_bids");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Winner {
    pub bidder: Addr,
    pub amount_to_pay: u128,
}
