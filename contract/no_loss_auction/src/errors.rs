use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AuctionError {
    DeadlineInPast = 1,
    InvalidStartingPrice = 2,
    AuctionNotFound = 3,
    AuctionNotActive = 4,
    AuctionAlreadyFinalized = 5,
    AuctionCanceled = 6,
    AuctionAlreadyEnded = 7,
    BidTooLow = 8,
    NoRefundBalance = 9,
    AuctionStillActive = 10,
    CannotCancelWithBids = 11,
    Unauthorized = 12,
}
