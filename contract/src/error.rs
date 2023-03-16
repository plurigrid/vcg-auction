use cosmwasm_std::{StdError, Uint64};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("There is already an auction in progress.")]
    AuctionAlreadyInProgress {},

    #[error("Auction is not in progress")]
    AuctionNotInProgress {},

    #[error("There must be at least 2 participants")]
    TooFewParticipants {},

    #[error("Bid amount must be greater than 0")]
    BidAmountTooLow {},

    #[error("Participant has already placed a bid for this auction")]
    BidAlreadyPlaced {},

    #[error("Not all bidders have cast their bids")]
    NotAllBiddersHaveBid {},

    #[error("No bids were found")]
    NoBidsFound {},

    #[error("Max number of auction participants has already been reached")]
    MaxParticipantsReached { max_participants: Uint64 },

    #[error("Auction not found")]
    AuctionNotFound { auction_id: u64 },
}
