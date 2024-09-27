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
        use frame_support::traits::Currency;
        use frame_support::sp_runtime::traits::Zero;
        use sp_core::H256;
        use scale_info::TypeInfo;
        use scale_info::prelude::fmt;
        use pallet_nft::{Pallet as NftPallet, NFTOwners};
        type MaxNftsLength = ConstU32<10000>;
        type MaxOfferNftsLength = ConstU32<10>;
        type NftItem = (H256, u32);
        pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


        /// The module configuration trait.
        #[pallet::config]
        pub trait Config: frame_system::Config + pallet_nft::Config + TypeInfo + fmt::Debug {
            type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
            type Currency: frame_support::traits::Currency<Self::AccountId>;
        }

        #[pallet::pallet]
        pub struct Pallet<T>(_);

        #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
        pub struct Offer<T: Config> {
            pub offered_nfts: BoundedVec<NftItem, MaxOfferNftsLength>,
            pub token_amount: BalanceOf<T>,
        }

        #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
        pub struct ListInfo<T: Config> {
            pub owner: T::AccountId,
            pub price: BalanceOf<T>,
        }

        /// The listed NFTs and their owners
        #[pallet::storage]
        pub type Listings<T: Config> = StorageMap<
            _,
            Twox64Concat,
            NftItem,
            ListInfo<T>,
        >;

        /// Offer for listed NFTs
        #[pallet::storage]
        pub type Offers<T: Config> = StorageMap<
            _,
            Twox64Concat,
            NftItem,
            BoundedVec<Offer<T>, MaxNftsLength>,
        >;

        #[pallet::event]
        #[pallet::generate_deposit(pub(super) fn deposit_event)]
        pub enum Event<T: Config> {
            /// An NFT was listed.
            NftListed(T::AccountId, NftItem),
            /// An NFT was unlisted.
            NftUnlisted(T::AccountId, NftItem),
            /// Buy NFT success.
            BuySuccess(NftItem, T::AccountId, BalanceOf<T>),
            /// An NFT offer was palced.
            OfferPlaced(NftItem, T::AccountId, Offer<T>),
            /// An NFT offer was palced.
            OfferCanceled(NftItem, T::AccountId, Offer<T>),
            /// An NFT offer was accepted.
            OfferAccepted(T::AccountId, NftItem, T::AccountId, Offer<T>),
            /// An NFT offer was rejected.
            OfferRejected(T::AccountId, NftItem, T::AccountId, Offer<T>),
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
            /// Token amount is insufficient.
            InsufficientBalance,
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
            pub fn list_nft(origin: OriginFor<T>, nft_item: NftItem, price: BalanceOf<T>) -> DispatchResult {
                let sender = ensure_signed(origin)?;
                let owner = NFTOwners::<T>::get(nft_item).ok_or(Error::<T>::NFTNotFound)?;
                ensure!(owner == sender, Error::<T>::NotOwner);

                let list_info = ListInfo {
                    owner: sender.clone(),
                    price,
                };
                Listings::<T>::insert(nft_item, &list_info);

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

            /// Buy an NFT.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT to buy.
            ///
            /// Emits `BuySucess` event when successful.
            #[pallet::call_index(2)]
            #[pallet::weight({0})]
            pub fn buy_nft(origin: OriginFor<T>,
                           nft_item: NftItem) -> DispatchResult {
                let buyer = ensure_signed(origin.clone())?;
                let list_info = Listings::<T>::get(nft_item).ok_or(Error::<T>::NotListed)?;
                let seller = list_info.owner;
                let buyer_balance = T::Currency::free_balance(&buyer.clone());

                ensure!(buyer_balance >= list_info.price, Error::<T>::InsufficientBalance);
                T::Currency::transfer(&buyer.clone(), &seller.clone(), list_info.price, frame_support::traits::ExistenceRequirement::AllowDeath)?;
                let seller_origin = frame_system::RawOrigin::Signed(seller.clone()).into();
                NftPallet::<T>::transfer_nft(seller_origin, buyer.clone(), nft_item)?;

                Listings::<T>::remove(nft_item);

                Self::deposit_event(Event::BuySuccess(nft_item, seller, list_info.price));
                Ok(())
            }

            /// Provide an offer to buy an NFT.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT to be purchased.
            /// - `offer_nfts`: The NFTs that needs to be used as an offer.
            /// - `token_amount`: The token amount that needs to be used as an offer.
            ///
            /// Emits `OfferPlaced` event when successful.
            #[pallet::call_index(3)]
            #[pallet::weight({0})]
            pub fn place_offer(origin: OriginFor<T>,
                               nft_item: NftItem,
                               offered_nfts: BoundedVec<NftItem, MaxOfferNftsLength>,
                               token_amount: BalanceOf<T>) -> DispatchResult {
                let sender = ensure_signed(origin)?;
                ensure!(Listings::<T>::contains_key(nft_item), Error::<T>::NotListed);

                for offered_nft_item in offered_nfts.clone().into_iter() {
                    let owner = NFTOwners::<T>::get(offered_nft_item).ok_or(Error::<T>::NFTNotFound)?;
                    ensure!(owner == sender, Error::<T>::NotOwner);
                }

                let offer_item = Offer {
                    offered_nfts,
                    token_amount,
                };

                Offers::<T>::mutate(nft_item, |offer_items| {
                    if offer_items.is_none() {
                        *offer_items = Some(BoundedVec::<Offer<T>, MaxNftsLength>::default());
                    }
                    if let Some(offer_items_value) = offer_items {
                        offer_items_value.try_push(offer_item.clone()).unwrap_or_default();
                    }
                });

                Self::deposit_event(Event::OfferPlaced(nft_item, sender, offer_item));
                Ok(())
            }

            /// Cancel an NFT offer.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT to be purchased.
            /// - `offer_nfts`: The NFTs that needs to be used as an offer.
            /// - `token_amount`: The token amount that needs to be used as an offer.
            ///
            /// Emits `OfferCanceled` event when successful.
            #[pallet::call_index(4)]
            #[pallet::weight({0})]
            pub fn cancel_offer(origin: OriginFor<T>,
                                nft_item: NftItem,
                                offered_nfts: BoundedVec<NftItem, MaxOfferNftsLength>,
                                token_amount: BalanceOf<T>) -> DispatchResult {
                let sender = ensure_signed(origin)?;
                ensure!(Listings::<T>::contains_key(nft_item), Error::<T>::NotListed);

                for offered_nft_item in offered_nfts.clone().into_iter() {
                    let owner = NFTOwners::<T>::get(offered_nft_item).ok_or(Error::<T>::NFTNotFound)?;
                    ensure!(owner == sender, Error::<T>::NotOwner);
                }

                let offer_item = Offer {
                    offered_nfts,
                    token_amount,
                };

                Offers::<T>::mutate(nft_item, |offer_items| {
                    if let Some(offer_items_value) = offer_items {
                        if let Some(index) = offer_items_value.iter().position(|x| *x == offer_item) {
                            offer_items_value.remove(index);
                        } else {
                            return Err(Error::<T>::NotOffered);
                        }
                    }
                    Ok(())
                })?;

                Self::deposit_event(Event::OfferCanceled(nft_item, sender, offer_item));
                Ok(())
            }

            /// Accept an offer.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT for sale.
            /// - `offered_nfts`: The NFTs that needs to be used as an offer.
            /// - `offered_token_amount`: The token amount that needs to be used as an offer.
            ///
            /// Emits `OfferAccepted` event when successful.
            #[pallet::call_index(5)]
            #[pallet::weight({0})]
            pub fn accept_offer(origin: OriginFor<T>,
                                nft_item: NftItem,
                                offered_nfts: BoundedVec<NftItem, MaxOfferNftsLength>,
                                offered_token_amount: BalanceOf<T>) -> DispatchResult {
                let sender = ensure_signed(origin.clone())?;
                let list_info = Listings::<T>::get(nft_item).ok_or(Error::<T>::NotListed)?;
                let seller = list_info.owner;
                ensure!(seller == sender, Error::<T>::NotOwner);

                let mut buyer_wrapper: Option<T::AccountId> = None;
                for offered_nft_item in offered_nfts.clone().into_iter() {
                    let owner = NFTOwners::<T>::get(offered_nft_item).ok_or(Error::<T>::NFTNotFound)?;
                    if buyer_wrapper.is_none() {
                        buyer_wrapper = Some(owner.clone());
                    } else if buyer_wrapper.as_ref() != Some(&owner) {
                        return Err(Error::<T>::NotOwner.into());
                    }
                }

                if let Some(buyer) = buyer_wrapper {
                    let offer_item = Offer {
                        offered_nfts: offered_nfts.clone(),
                        token_amount: offered_token_amount
                    };

                    let offers = Offers::<T>::get(nft_item).ok_or(Error::<T>::NotOffered)?;
                    let mut found_offer = false;
                    for item in offers.iter() {
                        if *item == offer_item {
                            found_offer = true;
                        }
                    }
                    ensure!(found_offer, Error::<T>::NotOffered);

                    NftPallet::<T>::transfer_nft(origin.clone(), buyer.clone(), nft_item)?;
                    for offered_nft_item in offered_nfts.clone().into_iter() {
                        NftPallet::<T>::transfer_nft(OriginFor::<T>::signed(buyer.clone()), seller.clone(), offered_nft_item)?;
                    }

                    if offered_token_amount > BalanceOf::<T>::zero() {
                       let buyer_balance = T::Currency::free_balance(&buyer.clone());
                       ensure!(buyer_balance >= offered_token_amount, Error::<T>::InsufficientBalance);
                       T::Currency::transfer(&buyer.clone(), &seller.clone(), offered_token_amount, frame_support::traits::ExistenceRequirement::AllowDeath)?;
                    }

                    Listings::<T>::remove(nft_item);
                    Offers::<T>::remove(nft_item);

                    Self::deposit_event(Event::OfferAccepted(seller, nft_item, buyer, offer_item));
                } else {
                    return Err(Error::<T>::NotOffered.into());
                }

                Ok(())
            }

            /// Reject an offer.
            ///
            /// The origin must be signed.
            ///
            /// Parameters:
            /// - `nft_item`: The NFT for sale.
            /// - `offered_nfts`: The NFTs that needs to be used as an offer.
            /// - `offered_token_amount`: The token amount that needs to be used as an offer.
            ///
            /// Emits `OfferRejected` event when successful.
            #[pallet::call_index(6)]
            #[pallet::weight({0})]
            pub fn reject_offer(origin: OriginFor<T>,
                                nft_item: NftItem,
                                offered_nfts: BoundedVec<NftItem, MaxOfferNftsLength>,
                                offered_token_amount: BalanceOf<T>) -> DispatchResult {
                let sender = ensure_signed(origin.clone())?;
                let list_info = Listings::<T>::get(nft_item).ok_or(Error::<T>::NotListed)?;
                let seller = list_info.owner;
                ensure!(seller == sender, Error::<T>::NotOwner);

                let mut buyer_wrapper: Option<T::AccountId> = None;
                for offered_nft_item in offered_nfts.clone().into_iter() {
                    let owner = NFTOwners::<T>::get(offered_nft_item).ok_or(Error::<T>::NFTNotFound)?;
                    if buyer_wrapper.is_none() {
                        buyer_wrapper = Some(owner.clone());
                    } else if buyer_wrapper.as_ref() != Some(&owner) {
                        return Err(Error::<T>::NotOwner.into());
                    }
                }

                if let Some(buyer) = buyer_wrapper {
                    let offer_item = Offer {
                        offered_nfts: offered_nfts.clone(),
                        token_amount: offered_token_amount,
                    };

                    Offers::<T>::mutate(nft_item, |offer_items| {
                        if let Some(offer_items_value) = offer_items {
                            if let Some(index) = offer_items_value.iter().position(|x| *x == offer_item) {
                                offer_items_value.remove(index);
                            } else {
                                return Err(Error::<T>::NotOffered);
                            }
                        }
                        Ok(())
                    })?;

                    Self::deposit_event(Event::OfferRejected(seller, nft_item, buyer, offer_item));
                } else {
                    return Err(Error::<T>::NotOffered.into());
                }

                Ok(())
            }
        }
}
