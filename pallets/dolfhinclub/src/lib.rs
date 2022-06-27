#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::Error::{MemberLimitExceeded, MemberNotFound};
	use codec;
	use frame_support::storage::bounded_vec::BoundedVec;
	use frame_support::{
		dispatch::{DispatchResult, PartialEq},
		pallet_prelude::*,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MAX: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_clubMember)]
	pub type ClubMember<T: Config> =
		StorageValue<_, BoundedVec<DolfhinMembers<T>, T::MAX>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_club_house)]
	pub type ClubHouse<T: Config> =
		StorageValue<_, BoundedVec<DolfhinMembers<T>, T::MAX>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_waitingList)]
	pub type WaitList<T: Config> = StorageValue<_, BoundedVec<T::AccountId, T::MAX>, ValueQuery>;

	#[derive(Encode, Decode, PartialEq, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct DolfhinMembers<T: Config> {
		AccountId: T::AccountId,
		RegisteredTime: T::BlockNumber,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		WaitListMemberAdded(T::AccountId),
		ClubMemberAdded(T::AccountId),
		ClubMemberRemoved(T::AccountId),
		MemberRemovedfromWaitList(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		MemberAlreadyExist,
		MemberNotFound,
		MemberLimitExceeded,
		CannotUnwrap,
		MemberAlreadyAClubMember,
		NotAMember,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10000)]
		pub fn add_wait_list(origin: OriginFor<T>) -> DispatchResult {
			let wait_member = ensure_signed(origin)?;

			ensure!(
				<WaitList<T>>::get().contains(&wait_member) == false,
				<Error<T>>::MemberAlreadyExist
			);

			<WaitList<T>>::try_mutate(|b_vec| b_vec.try_push(wait_member.clone()))
				.map_err(|_| <Error<T>>::MemberLimitExceeded)?;

			Self::deposit_event(Event::WaitListMemberAdded(wait_member));
			Ok(())
		}
		#[pallet::weight(10000)]
		pub fn add_member_to_club_one(
			origin: OriginFor<T>,
			member: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(Self::get_waitingList().contains(&member) == true, <Error<T>>::NotAMember);

			let time = <frame_system::Pallet<T>>::block_number();
			let mem = DolfhinMembers { AccountId: member.clone(), RegisteredTime: time };

			<ClubMember<T>>::try_mutate(|b_vec| b_vec.try_push(mem))
				.map_err(|_| <Error<T>>::MemberLimitExceeded)?;

			<WaitList<T>>::try_mutate(|b_vec| {
				if let Some(index) = b_vec.iter().position(|mem| *mem == member) {
					b_vec.remove(index);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::MemberNotFound)?;

			Self::deposit_event(Event::MemberRemovedfromWaitList(member.clone()));
			Self::deposit_event(Event::ClubMemberAdded(member.clone()));
			Ok(())
		}

		#[pallet::weight(10000)]
		pub fn remove_member(origin: OriginFor<T>, member: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			<ClubMember<T>>::try_mutate(|b_vec| {
				if let Some(index) = b_vec.iter().position(|mem| mem.AccountId == member) {
					b_vec.remove(index);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::MemberNotFound)?;
			Self::deposit_event(Event::ClubMemberRemoved(member.clone()));
			Ok(())
		}
	}
}
