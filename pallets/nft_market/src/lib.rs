#![cfg_attr(not(feature = "std"), no_std)]

/// A module for NFT Market
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
        use super::*;
        use frame_support::traits::OriginTrait;
        use frame_system::pallet_prelude::*;
        use frame_support::pallet_prelude::*;
        use sp_core::H256;
        use pallet_nft::{Pallet as NftPallet, NFTOwners};
        type MaxNftsLength = ConstU32<10000>;
        type NftItem = (H256, u32);

        /// The module configuration trait.
        #[pallet::config]
        pub trait Config: frame_system::Config + pallet_nft::Config {
                type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        }

        #[pallet::pallet]
        pub struct Pallet<T>(_);

        /// The listed NFTs and their owners
        #[pallet::storage]
        pub type Listings<T: Config> = StorageMap<
            _,
            Twox64Concat,
            NftItem,
            T::AccountId,
        >;

        /// Offer for listed NFTs
        #[pallet::storage]
        pub type Offers<T: Config> = StorageMap<
            _,
            Twox64Concat,
            NftItem,
            BoundedVec<NftItem, MaxNftsLength>, // collection, item_id
        >;

        #[pallet::event]
        #[pallet::generate_deposit(pub(super) fn deposit_event)]
        pub enum Event<T: Config> {
            /// An NFT was listed.
            NftListed(T::AccountId, NftItem),
            /// An NFT was unlisted.
            NftUnlisted(T::AccountId, NftItem),
            /// An NFT offer was palced.
            OfferPlaced(NftItem, T::AccountId, NftItem), // listed, offered
            /// An NFT offer was palced.
            OfferCanceled(NftItem, T::AccountId, NftItem), // listed, offered
            /// An NFT offer was accepted.
            OfferAccepted(T::AccountId, NftItem, T::AccountId, NftItem), // listed, offered
            /// An NFT offer was rejected.
            OfferRejected(T::AccountId, NftItem, T::AccountId, NftItem), // listed, offered
        }

        #[pallet::error]
        pub enum Error<T> {
            /// The NFT is not found.
            NFTNotFound,
            /// The owner of NFT is not the signed account.
            NotOwner,
            /// The NFT has been already listed.
            NftAlreadyListed,
            /// The NFT has not been listed.
            NotListed,
            /// The NFT offer has not been placed.
            NotOffered,
        }

        #[pallet::call]
        impl<T: Config> Pallet<T> {
            /// List an NFT so that others can buy it.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT to be listed.
            ///
            /// Emits `NftListed` event when successful.
            #[pallet::call_index(0)]
            #[pallet::weight({0})]
            pub fn list_nft(origin: OriginFor<T>, nft_item: NftItem) -> DispatchResult {
                let sender = ensure_signed(origin)?;
                let owner = NFTOwners::<T>::get(nft_item).ok_or(Error::<T>::NFTNotFound)?;
                ensure!(owner == sender, Error::<T>::NotOwner);

                Listings::<T>::insert(nft_item, sender.clone());

                Self::deposit_event(Event::NftListed(sender, nft_item));

                Ok(())
            }

            /// Unlist an NFT.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT to be unlisted.
            ///
            /// Emits `NftUnlisted` event when successful.
            #[pallet::call_index(1)]
            #[pallet::weight({0})]
            pub fn unlist_nft(origin: OriginFor<T>, nft_item: NftItem) -> DispatchResult {
                let sender = ensure_signed(origin)?;
                let owner = NFTOwners::<T>::get(nft_item).ok_or(Error::<T>::NFTNotFound)?;
                ensure!(owner == sender, Error::<T>::NotOwner);

                Listings::<T>::remove(nft_item);
                Offers::<T>::remove(nft_item);

                Self::deposit_event(Event::NftUnlisted(sender, nft_item));

                Ok(())
            }

            /// Provide an offer to buy an NFT.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT to be purchased.
            /// - `offer_nft_item`: The NFT that needs to be used as an offer.
            ///
            /// Emits `OfferPlaced` event when successful.
            #[pallet::call_index(2)]
            #[pallet::weight({0})]
            pub fn place_offer(origin: OriginFor<T>, nft_item: NftItem, offer_nft_item: NftItem) -> DispatchResult {
                let sender = ensure_signed(origin)?;
                ensure!(Listings::<T>::contains_key(nft_item), Error::<T>::NotListed);

                let owner = NFTOwners::<T>::get(offer_nft_item).ok_or(Error::<T>::NFTNotFound)?;
                ensure!(owner == sender, Error::<T>::NotOwner);

                Offers::<T>::mutate(nft_item, |nfts| {
                    if nfts.is_none() {
                        *nfts = Some(BoundedVec::<NftItem, MaxNftsLength>::default());
                    }
                    if let Some(nfts_value) = nfts {
                        nfts_value.try_push(offer_nft_item).unwrap_or_default();
                    }
                });

                Self::deposit_event(Event::OfferPlaced(nft_item, sender, offer_nft_item));
                Ok(())
            }

            /// Cancel an NFT offer.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT to be purchased.
            /// - `offer_nft_item`: The NFT that needs to be used as an offer.
            ///
            /// Emits `OfferCanceled` event when successful.
            #[pallet::call_index(3)]
            #[pallet::weight({0})]
            pub fn cancel_offer(origin: OriginFor<T>, nft_item: NftItem, offer_nft_item: NftItem) -> DispatchResult {
                let sender = ensure_signed(origin)?;
                ensure!(Listings::<T>::contains_key(nft_item), Error::<T>::NotListed);

                let owner = NFTOwners::<T>::get(offer_nft_item).ok_or(Error::<T>::NFTNotFound)?;
                ensure!(owner == sender, Error::<T>::NotOwner);

                //let mut offers = Offers::<T>::get(nft_item).ok_or(Error::<T>::NotOffered)?;
                //if let Some(index) = offers.iter().position(|&x| x == offer_nft_item) {
                //    offers.remove(index);
                //} else {
                //    return Err(Error::<T>::NotOffered.into());
                //}

                Offers::<T>::mutate(nft_item, |offer_items| {
                    if let Some(offer_items_value) = offer_items {
                        if let Some(index) = offer_items_value.iter().position(|&x| x == offer_nft_item) {
                            offer_items_value.remove(index);
                        } else {
                            return Err(Error::<T>::NotOffered);
                        }
                    }
                    Ok(())
                })?;

                Self::deposit_event(Event::OfferCanceled(nft_item, sender, offer_nft_item));
                Ok(())
            }

            /// Accept an offer.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT for sale.
            /// - `offer_nft_item`: The NFT that used as an offer.
            ///
            /// Emits `OfferAccepted` event when successful.
            #[pallet::call_index(4)]
            #[pallet::weight({0})]
            pub fn accept_offer(origin: OriginFor<T>, nft_item: NftItem, offer_nft_item: NftItem) -> DispatchResult {
                let sender = ensure_signed(origin.clone())?;
                let seller = Listings::<T>::get(nft_item).ok_or(Error::<T>::NotListed)?;
                ensure!(seller == sender, Error::<T>::NotOwner);

                let buyer = NFTOwners::<T>::get(offer_nft_item).ok_or(Error::<T>::NFTNotFound)?;
                let offers = Offers::<T>::get(nft_item).ok_or(Error::<T>::NotOffered)?;

                let mut found_offer = false;
                for &item in offers.iter() {
                    if item == offer_nft_item {
                        found_offer = true;
                    }
                }
                ensure!(found_offer, Error::<T>::NotOffered);

                NftPallet::<T>::transfer_nft(origin.clone(), buyer.clone(), nft_item)?;
                NftPallet::<T>::transfer_nft(OriginFor::<T>::signed(buyer.clone()), seller.clone(), offer_nft_item)?;

                Listings::<T>::remove(nft_item);
                Offers::<T>::remove(nft_item);

                Self::deposit_event(Event::OfferAccepted(seller, nft_item, buyer, offer_nft_item));

                Ok(())
            }

            /// Reject an offer.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT for sale.
            /// - `offer_nft_item`: The NFT that used as an offer.
            ///
            /// Emits `OfferRejected` event when successful.
            #[pallet::call_index(5)]
            #[pallet::weight({0})]
            pub fn reject_offer(origin: OriginFor<T>, nft_item: NftItem, offer_nft_item: NftItem) -> DispatchResult {
                let sender = ensure_signed(origin.clone())?;
                let seller = Listings::<T>::get(nft_item).ok_or(Error::<T>::NotListed)?;
                ensure!(seller == sender, Error::<T>::NotOwner);

                let buyer = NFTOwners::<T>::get(offer_nft_item).ok_or(Error::<T>::NFTNotFound)?;
                Offers::<T>::mutate(nft_item, |offer_items| {
                    if let Some(offer_items_value) = offer_items {
                        if let Some(index) = offer_items_value.iter().position(|&x| x == offer_nft_item) {
                            offer_items_value.remove(index);
                        } else {
                            return Err(Error::<T>::NotOffered);
                        }
                    }
                    Ok(())
                })?;

                Self::deposit_event(Event::OfferRejected(seller, nft_item, buyer, offer_nft_item));

                Ok(())
            }
        }
}
