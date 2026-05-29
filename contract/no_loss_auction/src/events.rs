use soroban_sdk::{Address, Env, Symbol, String};

pub fn auction_created(env: &Env, auction_id: u32, seller: Address, title: String) {
    env.events()
        .publish((Symbol::new(env, "auction_created"), auction_id), (seller, title));
}

pub fn bid_placed(env: &Env, auction_id: u32, bidder: Address, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "bid_placed"), auction_id), (bidder, amount));
}

pub fn refund_created(env: &Env, auction_id: u32, bidder: Address, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "refund_created"), auction_id), (bidder, amount));
}

pub fn refund_claimed(env: &Env, user: Address, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "refund_claimed"), user.clone()), (user, amount));
}

pub fn auction_finalized(env: &Env, auction_id: u32, winner: Option<Address>, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "auction_finalized"), auction_id), (winner, amount));
}

pub fn auction_canceled(env: &Env, auction_id: u32) {
    env.events()
        .publish((Symbol::new(env, "auction_canceled"), auction_id), ());
}
