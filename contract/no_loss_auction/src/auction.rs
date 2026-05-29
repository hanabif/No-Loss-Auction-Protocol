use soroban_sdk::{
    contract, contractimpl, token, Address, Env, String,
};

use crate::errors::AuctionError;
use crate::events;
use crate::storage::{Auction, DataKey};

#[contract]
pub struct NoLossAuction;

#[contractimpl]
impl NoLossAuction {
    /// Creates a new auction
    pub fn create_auction(
        env: Env,
        seller: Address,
        token: Address,
        title: String,
        starting_price: i128,
        deadline: u64,
    ) -> Result<u32, AuctionError> {
        seller.require_auth();

        if starting_price <= 0 {
            return Err(AuctionError::InvalidStartingPrice);
        }

        if deadline <= env.ledger().timestamp() {
            return Err(AuctionError::DeadlineInPast);
        }

        let mut count: u32 = env.storage().instance().get(&DataKey::AuctionCounter).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::AuctionCounter, &count);

        let auction = Auction {
            seller: seller.clone(),
            token,
            title: title.clone(),
            starting_price,
            highest_bid: starting_price,
            highest_bidder: None,
            deadline,
            active: true,
            finalized: false,
            canceled: false,
            bid_count: 0,
        };

        env.storage().persistent().set(&DataKey::Auction(count), &auction);

        events::auction_created(&env, count, seller, title);

        Ok(count)
    }

    /// Places a bid in an auction using a SEP-41 token
    pub fn place_bid(
        env: Env,
        auction_id: u32,
        bidder: Address,
        amount: i128,
    ) -> Result<(), AuctionError> {
        bidder.require_auth();

        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .ok_or(AuctionError::AuctionNotFound)?;

        if !auction.active || auction.canceled || auction.finalized {
            return Err(AuctionError::AuctionNotActive);
        }

        if env.ledger().timestamp() >= auction.deadline {
            return Err(AuctionError::AuctionAlreadyEnded);
        }

        if amount <= auction.highest_bid {
            return Err(AuctionError::BidTooLow);
        }

        // Transfer funds to contract
        let token_client = token::Client::new(&env, &auction.token);
        token_client.transfer(&bidder, &env.current_contract_address(), &amount);

        // Record refund out for the previous highest bidder
        if let Some(prev_bidder) = auction.highest_bidder {
            let refund_key = DataKey::Refund(prev_bidder.clone());
            let current_refund: i128 = env.storage().persistent().get(&refund_key).unwrap_or(0);
            env.storage().persistent().set(&refund_key, &(current_refund + auction.highest_bid));
            events::refund_created(&env, auction_id, prev_bidder, auction.highest_bid);
        }

        auction.highest_bid = amount;
        auction.highest_bidder = Some(bidder.clone());
        auction.bid_count += 1;

        env.storage().persistent().set(&DataKey::Auction(auction_id), &auction);

        events::bid_placed(&env, auction_id, bidder, amount);

        Ok(())
    }

    /// Claim refund from previous outbids
    pub fn claim_refund(env: Env, user: Address, token: Address) -> Result<(), AuctionError> {
        user.require_auth();

        let refund_key = DataKey::Refund(user.clone());
        let amount: i128 = env.storage().persistent().get(&refund_key).unwrap_or(0);

        if amount <= 0 {
            return Err(AuctionError::NoRefundBalance);
        }

        // Reset balance before transferring to mitigate re-entrancy, though Soroban handles this well
        env.storage().persistent().set(&refund_key, &0i128);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &user, &amount);

        events::refund_claimed(&env, user, amount);

        Ok(())
    }

    /// Finalize the auction and transfer funds to the seller
    pub fn finalize_auction(env: Env, auction_id: u32) -> Result<(), AuctionError> {
        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .ok_or(AuctionError::AuctionNotFound)?;

        if !auction.active {
            return Err(AuctionError::AuctionNotActive);
        }

        if auction.finalized {
            return Err(AuctionError::AuctionAlreadyFinalized);
        }

        if env.ledger().timestamp() < auction.deadline {
            return Err(AuctionError::AuctionStillActive);
        }

        auction.active = false;
        auction.finalized = true;

        if let Some(winner) = auction.highest_bidder.clone() {
            let token_client = token::Client::new(&env, &auction.token);
            token_client.transfer(&env.current_contract_address(), &auction.seller, &auction.highest_bid);
            events::auction_finalized(&env, auction_id, Some(winner), auction.highest_bid);
        } else {
            events::auction_finalized(&env, auction_id, None, 0);
        }

        env.storage().persistent().set(&DataKey::Auction(auction_id), &auction);

        Ok(())
    }

    /// Cancel an auction if no bids were placed
    pub fn cancel_auction(env: Env, auction_id: u32, seller: Address) -> Result<(), AuctionError> {
        seller.require_auth();

        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .ok_or(AuctionError::AuctionNotFound)?;

        if auction.seller != seller {
            return Err(AuctionError::Unauthorized);
        }

        if !auction.active || auction.canceled {
            return Err(AuctionError::AuctionNotActive);
        }

        if auction.bid_count > 0 {
            return Err(AuctionError::CannotCancelWithBids);
        }

        auction.active = false;
        auction.canceled = true;

        env.storage().persistent().set(&DataKey::Auction(auction_id), &auction);

        events::auction_canceled(&env, auction_id);

        Ok(())
    }

    /// Get details of an auction
    pub fn get_auction(env: Env, auction_id: u32) -> Option<Auction> {
        env.storage().persistent().get(&DataKey::Auction(auction_id))
    }

    /// Get refund balance of a user
    pub fn get_refund_balance(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Refund(user)).unwrap_or(0)
    }
}
