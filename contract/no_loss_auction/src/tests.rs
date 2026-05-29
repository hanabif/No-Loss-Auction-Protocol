#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String,
};

use crate::{NoLossAuction, NoLossAuctionClient};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> (Address, token::Client<'a>) {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = token::Client::new(env, &token_address.address());
    (token_address.address(), token_client)
}

#[test]
fn test_create_auction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(NoLossAuction, ());
    let client = NoLossAuctionClient::new(&env, &contract_id);
    
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_id, _) = create_token_contract(&env, &token_admin);
    
    let title = String::from_str(&env, "Test Auction");
    let starting_price = 100;
    
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let deadline = 2000;
    
    let auction_id = client.create_auction(&seller, &token_id, &title, &starting_price, &deadline);
    assert_eq!(auction_id, 1);
    
    let auction = client.get_auction(&auction_id).unwrap();
    assert_eq!(auction.seller, seller);
    assert_eq!(auction.starting_price, 100);
    assert_eq!(auction.highest_bid, 100);
    assert_eq!(auction.highest_bidder, None);
    assert_eq!(auction.deadline, 2000);
    assert_eq!(auction.active, true);
}

#[test]
fn test_place_bid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(NoLossAuction, ());
    let client = NoLossAuctionClient::new(&env, &contract_id);
    
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    let token_admin = Address::generate(&env);
    
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address.address());
    let token_client = token::Client::new(&env, &token_address.address());
    
    token_admin_client.mint(&bidder, &1000);
    
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let title = String::from_str(&env, "Test Auction");
    
    let auction_id = client.create_auction(&seller, &token_address.address(), &title, &100, &2000);
    
    client.place_bid(&auction_id, &bidder, &150);
    
    let auction = client.get_auction(&auction_id).unwrap();
    assert_eq!(auction.highest_bid, 150);
    assert_eq!(auction.highest_bidder, Some(bidder.clone()));
    
    assert_eq!(token_client.balance(&bidder), 850);
    assert_eq!(token_client.balance(&contract_id), 150);
}

#[test]
fn test_refund_system() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(NoLossAuction, ());
    let client = NoLossAuctionClient::new(&env, &contract_id);
    
    let seller = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    let token_admin = Address::generate(&env);
    
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address.address());
    let token_client = token::Client::new(&env, &token_address.address());
    
    token_admin_client.mint(&bidder1, &1000);
    token_admin_client.mint(&bidder2, &1000);
    
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let title = String::from_str(&env, "Test Auction");
    
    let auction_id = client.create_auction(&seller, &token_address.address(), &title, &100, &2000);
    
    client.place_bid(&auction_id, &bidder1, &150);
    client.place_bid(&auction_id, &bidder2, &200);
    
    let refund_balance = client.get_refund_balance(&bidder1);
    assert_eq!(refund_balance, 150);
    
    client.claim_refund(&bidder1, &token_address.address());
    assert_eq!(client.get_refund_balance(&bidder1), 0);
    
    assert_eq!(token_client.balance(&bidder1), 1000); // 1000 - 150 + 150
}

#[test]
fn test_finalize_auction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(NoLossAuction, ());
    let client = NoLossAuctionClient::new(&env, &contract_id);
    
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    let token_admin = Address::generate(&env);
    
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address.address());
    let token_client = token::Client::new(&env, &token_address.address());
    
    token_admin_client.mint(&bidder, &1000);
    
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let title = String::from_str(&env, "Test Auction");
    
    let auction_id = client.create_auction(&seller, &token_address.address(), &title, &100, &2000);
    
    client.place_bid(&auction_id, &bidder, &150);
    
    env.ledger().with_mut(|li| li.timestamp = 2500);
    client.finalize_auction(&auction_id);
    
    let auction = client.get_auction(&auction_id).unwrap();
    assert_eq!(auction.finalized, true);
    assert_eq!(auction.active, false);
    
    assert_eq!(token_client.balance(&seller), 150);
}

#[test]
fn test_cancel_auction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register(NoLossAuction, ());
    let client = NoLossAuctionClient::new(&env, &contract_id);
    
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_id, _) = create_token_contract(&env, &token_admin);
    
    env.ledger().with_mut(|li| li.timestamp = 1000);
    let title = String::from_str(&env, "Test Auction");
    
    let auction_id = client.create_auction(&seller, &token_id, &title, &100, &2000);
    
    client.cancel_auction(&auction_id, &seller);
    
    let auction = client.get_auction(&auction_id).unwrap();
    assert_eq!(auction.canceled, true);
    assert_eq!(auction.active, false);
}
