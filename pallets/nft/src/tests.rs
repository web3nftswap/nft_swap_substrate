use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_core::H256;
use sp_core::hashing::blake2_256;

type AccountId = <Test as frame_system::Config>::AccountId;

#[test]
fn create_collections() {
    new_test_ext().execute_with(|| {
        let account_id: AccountId = 1;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id), max_items, metainfo.clone()));
        
        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        assert_eq!(
            NFTCollections::<Test>::get(&collection_id),
            Some((max_items, 0_u32, metainfo))
        );

        let collection_id_boundedvec = BoundedVec::try_from(vec![collection_id]).unwrap();
        assert_eq!(NFTCollectionIds::<Test>::get(), Some(collection_id_boundedvec));
    })
}

#[test]
fn create_collections_fail_when_already_exist() {
    new_test_ext().execute_with(|| {
        let account_id: AccountId = 1;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id), max_items, metainfo.clone()));
        assert_noop!(
            NftModule::create_collection(RuntimeOrigin::signed(account_id), max_items, metainfo.clone()),
            Error::<Test>::CollectionAlreadyExists
        );
    })
}

#[test]
fn mint() {
    new_test_ext().execute_with(|| {
        let account_id: AccountId = 1;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id), max_items, metainfo.clone()));

        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id), collection_id, metainfo1.clone()));
        let mut items_boundedvec = BoundedVec::try_from(vec![(collection_id, 0)]).unwrap();
        assert_eq!(OwnedNFTs::<Test>::get(account_id), Some(items_boundedvec));
        items_boundedvec = BoundedVec::try_from(vec![(collection_id, 0), (collection_id, 1)]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id), collection_id, metainfo1.clone()));
        assert_eq!(OwnedNFTs::<Test>::get(account_id), Some(items_boundedvec));

        assert_eq!(NFTOwners::<Test>::get((collection_id, 0)), Some(account_id));
        assert_eq!(NFTOwners::<Test>::get((collection_id, 1)), Some(account_id));
    })
}

#[test]
fn mint_fail_when_exceed() {
    new_test_ext().execute_with(|| {
        let account_id: AccountId = 1;
        let max_items: u32 = 2;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id), max_items, metainfo.clone()));

        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id), collection_id, metainfo1.clone()));
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id), collection_id, metainfo1.clone()));
        assert_noop!(
            NftModule::mint_nft(RuntimeOrigin::signed(account_id), collection_id, metainfo1.clone()),
            Error::<Test>::NFTExceeds
        );
    })
}

#[test]
fn transfer() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));

        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));
        let metainfo1 = BoundedVec::try_from(vec![1, 2]).unwrap();
        assert_ok!(NftModule::mint_nft(RuntimeOrigin::signed(account_id0), collection_id, metainfo1.clone()));
        assert_eq!(NFTOwners::<Test>::get((collection_id, 0)), Some(account_id0));

        assert_ok!(NftModule::transfer_nft(RuntimeOrigin::signed(account_id0), account_id1, (collection_id, 0)));
        assert_eq!(NFTOwners::<Test>::get((collection_id, 0)), Some(account_id1));
    })
}

#[test]
fn transfer_when_nft_not_owned() {
    new_test_ext().execute_with(|| {
        let account_id0: AccountId = 1;
        let account_id1: AccountId = 2;
        let max_items: u32 = 100;
        let metainfo = BoundedVec::try_from(vec![0, 1]).unwrap();
        assert_ok!(NftModule::create_collection(RuntimeOrigin::signed(account_id0), max_items, metainfo.clone()));

        let collection_id = H256::from_slice(&blake2_256(&metainfo.clone()));

        assert_noop!(
            NftModule::transfer_nft(RuntimeOrigin::signed(account_id0), account_id1, (collection_id, 0)),
            Error::<Test>::NFTNotFound
        );
    })
}

