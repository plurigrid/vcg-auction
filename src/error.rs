use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Auction is in progress")]
    AuctionAlreadyStarted {},

    #[error("Auction not started")]
    AuctionNotStarted {},

    #[error("There must be at least 2 participants")]
    TooFewParticipants {},

    #[error("Bid amount must be greater than 0")]
    BidAmountTooLow {},

    #[error("Participant has already bid")]
    ParticipantAlreadyBid {},

    #[error("Not all bidders have cast their bids")]
    NotAllBiddersHaveBid {},

    #[error("No bids were found")]
    NoBidsFound {},
}
