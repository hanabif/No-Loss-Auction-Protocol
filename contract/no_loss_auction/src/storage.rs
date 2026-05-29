use soroban_sdk::{contracttype, Address, String};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Auction(u32),
    AuctionCounter,
    Refund(Address),
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct Auction {
    pub seller: Address,
    pub token: Address,
    pub title: String,
    pub starting_price: i128,
    pub highest_bid: i128,
    pub highest_bidder: Option<Address>,
    pub deadline: u64,
    pub active: bool,
    pub finalized: bool,
    pub canceled: bool,
    pub bid_count: u32,
}
