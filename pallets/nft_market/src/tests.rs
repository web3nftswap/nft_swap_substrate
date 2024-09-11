use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_core::H256;
use sp_core::hashing::blake2_256;
use pallet_nft::NFTOwners;

type AccountId = <Test as frame_system::Config>::AccountId;

#[test]
fn list_nft() {
    new_test_ext().execute_with(|| {
        let account_id: AccountId = 1;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id), collection_id, metainfo1.clone()));

        assert_ok!(NftMarketModule::list_nft(RuntimeOrigin::signed(account_id), (collection_id, 0)));
        assert_eq!(Listings::<Test>::get((collection_id, 0)), Some(account_id));
    })
}

#[test]
fn list_nft_fail_when_not_owner() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));

        assert_noop!(
            NftMarketModule::list_nft(RuntimeOrigin::signed(account_id1), (collection_id, 0)),
            Error::<Test>::NotOwner
        );

        assert_noop!(
            NftMarketModule::list_nft(RuntimeOrigin::signed(account_id0), (collection_id, 1)),
            Error::<Test>::NFTNotFound
        );
    })
}

#[test]
fn unlist_nft() {
    new_test_ext().execute_with(|| {
        let account_id: AccountId = 1;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id), collection_id, metainfo1.clone()));

        let nft_item = (collection_id, 0);
        assert_ok!(NftMarketModule::list_nft(RuntimeOrigin::signed(account_id), nft_item));
        assert_eq!(Listings::<Test>::get(nft_item), Some(account_id));
        assert_ok!(NftMarketModule::unlist_nft(RuntimeOrigin::signed(account_id), nft_item));
        assert_eq!(Listings::<Test>::get(nft_item), None);
    })
}

#[test]
fn place_offer() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));
        assert_ok!(NftMarketModule::list_nft(RuntimeOrigin::signed(account_id0), (collection_id, 0)));

        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id1), collection_id, metainfo1.clone()));
        let token_amount: u128 = 10;
        let offer_nfts = BoundedVec::try_from(vec![(collection_id, 1)]).unwrap();
        assert_ok!(NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts, token_amount));

        let offered_nfts_boundedvec = BoundedVec::try_from(vec![(collection_id, 1)]).unwrap();
        let offer = Offer {
            offered_nfts: offered_nfts_boundedvec,
            token_amount
        };
        let offered_boundedvec = BoundedVec::try_from(vec![offer]).unwrap();
        assert_eq!(Offers::<Test>::get((collection_id, 0)), Some(offered_boundedvec));
    })
}

#[test]
fn place_offer_fail_with_wrong_nft() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));
        assert_ok!(NftMarketModule::list_nft(RuntimeOrigin::signed(account_id0), (collection_id, 0)));

        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id1), collection_id, metainfo1.clone()));

        // account0: nft0,1
        // account1: nft2
        let token_amount: u128 = 0;
        let offer_nfts_id2 = BoundedVec::try_from(vec![(collection_id, 2)]).unwrap();
        assert_ok!(NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts_id2.clone(), token_amount));

        assert_noop!(
            NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 1), offer_nfts_id2.clone(), token_amount),
            Error::<Test>::NotListed
        );

        let offer_nfts_id3 = BoundedVec::try_from(vec![(collection_id, 3)]).unwrap();
        assert_noop!(
            NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts_id3, token_amount),
            Error::<Test>::NFTNotFound
        );

        let offer_nfts_id1 = BoundedVec::try_from(vec![(collection_id, 1)]).unwrap();
        assert_noop!(
            NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts_id1, token_amount),
            Error::<Test>::NotOwner
        );
    })
}

#[test]
fn cancel_offer() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));
        assert_ok!(NftMarketModule::list_nft(RuntimeOrigin::signed(account_id0), (collection_id, 0)));

        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id1), collection_id, metainfo1.clone()));
        let token_amount: u128 = 0;
        let offer_nfts= BoundedVec::try_from(vec![(collection_id, 1)]).unwrap();
        assert_ok!(NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts.clone(), token_amount));
        assert_ok!(NftMarketModule::cancel_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts.clone(), token_amount));

        let offered_boundedvec = BoundedVec::try_from(vec![]).unwrap();
        assert_eq!(Offers::<Test>::get((collection_id, 0)), Some(offered_boundedvec));
    })
}

#[test]
fn accept_offer() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));
        assert_ok!(NftMarketModule::list_nft(RuntimeOrigin::signed(account_id0), (collection_id, 0)));

        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id1), collection_id, metainfo1.clone()));
        let token_amount: u128 = 20;
        let offer_nfts= BoundedVec::try_from(vec![(collection_id, 1)]).unwrap();
        assert_ok!(NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts.clone(), token_amount));
        assert_ok!(NftMarketModule::accept_offer(RuntimeOrigin::signed(account_id0), (collection_id, 0), offer_nfts.clone(), token_amount));

        assert_eq!(NFTOwners::<Test>::get((collection_id, 0)), Some(account_id1));
        assert_eq!(NFTOwners::<Test>::get((collection_id, 1)), Some(account_id0));
    })
}

#[test]
fn accept_offer_fail_with_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));
        assert_ok!(NftMarketModule::list_nft(RuntimeOrigin::signed(account_id0), (collection_id, 0)));

        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id1), collection_id, metainfo1.clone()));
        let token_amount: u128 = 200000000;
        let offer_nfts= BoundedVec::try_from(vec![(collection_id, 1)]).unwrap();
        assert_ok!(NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts.clone(), token_amount));
        assert_noop!(
            NftMarketModule::accept_offer(RuntimeOrigin::signed(account_id0), (collection_id, 0), offer_nfts.clone(), token_amount),
            Error::<Test>::InsufficientBalance
        );

        assert_eq!(NFTOwners::<Test>::get((collection_id, 0)), Some(account_id0));
        assert_eq!(NFTOwners::<Test>::get((collection_id, 1)), Some(account_id1));
    })
}

#[test]
fn reject_offer() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));
        assert_ok!(NftMarketModule::list_nft(RuntimeOrigin::signed(account_id0), (collection_id, 0)));

        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id1), collection_id, metainfo1.clone())); // id1
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id1), collection_id, metainfo1.clone())); // id2
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id1), collection_id, metainfo1.clone())); // id3
        let token_amount: u128 = 0;
        let offer_nfts_id1= BoundedVec::try_from(vec![(collection_id, 1)]).unwrap();
        assert_ok!(NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts_id1.clone(), token_amount));
        let offer_nfts_id23= BoundedVec::try_from(vec![(collection_id, 2), (collection_id, 3)]).unwrap();
        assert_ok!(NftMarketModule::place_offer(RuntimeOrigin::signed(account_id1), (collection_id, 0), offer_nfts_id23, token_amount));

        assert_ok!(NftMarketModule::reject_offer(RuntimeOrigin::signed(account_id0), (collection_id, 0), offer_nfts_id1.clone(), token_amount));

        let offered_nfts_boundedvec = BoundedVec::try_from(vec![(collection_id, 2), (collection_id, 3)]).unwrap();
        let offer = Offer {
            offered_nfts: offered_nfts_boundedvec,
            token_amount
        };
        let offered_boundedvec = BoundedVec::try_from(vec![offer]).unwrap();
        assert_eq!(Offers::<Test>::get((collection_id, 0)), Some(offered_boundedvec));
        assert_eq!(NFTOwners::<Test>::get((collection_id, 0)), Some(account_id0));
        assert_eq!(NFTOwners::<Test>::get((collection_id, 1)), Some(account_id1));
    })
}

