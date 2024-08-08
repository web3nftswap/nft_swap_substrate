#![cfg_attr(not(feature = "std"), no_std)]

/// A module for NFT
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
        use super::*;
        use frame_system::pallet_prelude::*;
        use sp_core::hashing::blake2_256;
        use sp_core::H256;
        use frame_support::pallet_prelude::*;
        type MaxMetadataLength = ConstU32<16>;
        type MaxCollectionsLength = ConstU32<100>;
        type MaxNftsLength = ConstU32<10000>;
        type NftItem = (H256, u32);

        #[pallet::config]
        pub trait Config: frame_system::Config {
                type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        }

        #[pallet::pallet]
        pub struct Pallet<T>(_);

        /// The collection id array.
        #[pallet::storage]
        pub type NFTCollectionIds<T: Config> = StorageValue<_, BoundedVec<H256, MaxCollectionsLength>>;

        /// The detail of a collection.
        #[pallet::storage]
        pub type NFTCollections<T: Config> = StorageMap<
            _,
            Blake2_128Concat,
            H256, // collection
            (u32, u32, BoundedVec<u8, MaxMetadataLength>), // (max_items, cur_item_index, collecton_metadata)
        >;

        /// The NFTs owned by an account. 
        #[pallet::storage]
        pub type OwnedNFTs<T: Config> = StorageMap<
            _,
            Blake2_128Concat,
            T::AccountId,
            BoundedVec<NftItem, MaxNftsLength>, // collection, item_id
        >;

        /// The metadata of an NFT.
        #[pallet::storage]
        pub type NFTMetadata<T: Config> = StorageMap<
            _,
            Blake2_128Concat,
            NftItem,
            BoundedVec<u8, MaxMetadataLength>, // nft_metadata
        >;

        /// The owner of an NFT.
        #[pallet::storage]
        pub type NFTOwners<T: Config> = StorageMap<
            _,
            Blake2_128Concat,
            NftItem,
            T::AccountId, // nft owner
        >;

        #[pallet::event]
        #[pallet::generate_deposit(pub(super) fn deposit_event)]
        pub enum Event<T: Config> {
            /// A collection was created.
            NFTCollectionCreated(T::AccountId, H256, u32), // account, collection, max_items
            /// An NFT was minted.
            NFTMinted(T::AccountId, NftItem),
            /// An NFT was transfered.
            NFTTransferred(T::AccountId, T::AccountId, NftItem),
        }

        #[pallet::error]
        pub enum Error<T> {
            /// The collection already exists.
            CollectionAlreadyExists,
            /// The collection is not found.
            CollectionNotFound,
            /// The number of collections has exceeded the maximum limit.
            CollectionExceeds,
            /// The NFT already exists.
            NFTAlreadyExists,
            /// The NFT is not found.
            NFTNotFound,
            /// The number of NFTs has exceeded the maximum limit.
            NFTExceeds,
            /// The owner of NFT is not the signed account.
            NotOwner,
        }

        #[pallet::call]
        impl<T: Config> Pallet<T> {
            /// Create an NFT collection.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `max_items`: The maximum NFT number of the collection.
            /// - `metadata`: The collection metadata.
            ///
            /// Emits `NFTCollectionCreated` event when successful.
            #[pallet::call_index(0)]
            #[pallet::weight({0})]
            pub fn create_collection(origin: OriginFor<T>, max_items: u32, metadata: BoundedVec<u8, MaxMetadataLength>) -> DispatchResult {
                let sender = ensure_signed(origin)?;
                let collection_id_array = blake2_256(&metadata);
                let collection_id = H256::from_slice(&collection_id_array);

                ensure!(!NFTCollections::<T>::contains_key(&collection_id), Error::<T>::CollectionAlreadyExists);

                NFTCollections::<T>::insert(&collection_id, (max_items, 0, metadata));
                NFTCollectionIds::<T>::mutate(|col| {
                    if col.is_none() {
                        *col = Some(BoundedVec::<H256, MaxCollectionsLength>::default());
                    }
                    if let Some(col_value) = col {
                        col_value.try_push(collection_id).unwrap_or_default();
                    }
                });

                Self::deposit_event(Event::NFTCollectionCreated(sender, collection_id, max_items));
                Ok(())
            }

            /// Mint an NFT.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `collection_id`: The collection id of an NFT.
            /// - `metadata`: The NFT metadata.
            ///
            /// Emits `NFTMinted` event when successful.
            #[pallet::call_index(1)]
            #[pallet::weight({0})]
            pub fn mint_nft(origin: OriginFor<T>, collection_id: H256, metadata: BoundedVec<u8, MaxMetadataLength>) -> DispatchResult {
                let sender = ensure_signed(origin)?;

                ensure!(NFTCollections::<T>::contains_key(&collection_id), Error::<T>::CollectionNotFound);

                let (max_items, cur_item_index, collection_metadata) = NFTCollections::<T>::get(&collection_id).ok_or(Error::<T>::CollectionNotFound)?;
                ensure!(cur_item_index < max_items, Error::<T>::NFTExceeds);

                let nft_item = (collection_id, cur_item_index);
                OwnedNFTs::<T>::mutate(&sender, |nfts| {
                    if nfts.is_none() {
                        *nfts = Some(BoundedVec::<NftItem, MaxNftsLength>::default());
                    }
                    if let Some(nfts_value) = nfts {
                        nfts_value.try_push(nft_item).unwrap_or_default();
                    }
                });

                NFTMetadata::<T>::insert(nft_item, metadata);
                NFTOwners::<T>::insert(nft_item, sender.clone());
                NFTCollections::<T>::insert(&collection_id, (max_items, cur_item_index + 1, collection_metadata));

                Self::deposit_event(Event::NFTMinted(sender, nft_item));
                Ok(())
            }

            /// Transfer an NFT.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `to`: The target account id for the transfer.
            /// - `nft_item`: The NFT to transfer.
            ///
            /// Emits `NFTTransferred` event when successful.
            #[pallet::call_index(2)]
            #[pallet::weight({0})]
            pub fn transfer_nft(origin: OriginFor<T>, to: T::AccountId, nft_item: NftItem) -> DispatchResult {
                let sender = ensure_signed(origin)?;

                ensure!(NFTMetadata::<T>::contains_key(nft_item), Error::<T>::NFTNotFound);

                let mut owned_nfts = OwnedNFTs::<T>::get(&sender).ok_or(Error::<T>::NFTNotFound)?;
                ensure!(owned_nfts.contains(&nft_item), Error::<T>::NotOwner);
                owned_nfts.retain(|&nft| nft != nft_item);
                OwnedNFTs::<T>::insert(&sender, owned_nfts);

                OwnedNFTs::<T>::mutate(&to, |nfts| {
                    if nfts.is_none() {
                        *nfts = Some(BoundedVec::<NftItem, MaxNftsLength>::default());
                    }
                    if let Some(nfts_value) = nfts {
                        nfts_value.try_push(nft_item).unwrap_or_default();
                    }
                });

                NFTOwners::<T>::insert(nft_item, to.clone());

                Self::deposit_event(Event::NFTTransferred(sender, to, nft_item));
                Ok(())
            }
        }
}
