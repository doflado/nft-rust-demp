#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use codec::FullCodec;
	use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub, Zero};
	use sp_std::vec::Vec;
	use sp_std::{
		cmp::{Eq, PartialEq},
		fmt::Debug,
	};
	use traits::Erc1155;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type TokenId: FullCodec
			+ Eq
			+ PartialEq
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ scale_info::TypeInfo;
		type Balance: AtLeast32BitUnsigned
			+ FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ Default
			+ scale_info::TypeInfo;
		#[pallet::constant]
		type Decimals: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_total_supply)]
	pub(super) type TotalSupply<T: Config> =
		StorageMap<_, Blake2_128Concat, T::TokenId, T::Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_balance)]
	pub(super) type Balances<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::TokenId,
		T::Balance,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_approval)]
	pub(super) type Approval<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		bool,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Initialized(T::AccountId, Vec<T::TokenId>, Vec<T::Balance>),
		TransferSingle(T::AccountId, T::AccountId, T::TokenId, T::Balance),
		TransferBatch(T::AccountId, T::AccountId, Vec<T::TokenId>, Vec<T::Balance>),
		ApprovalForAll(T::AccountId, T::AccountId, bool),
	}

	#[pallet::error]
	pub enum Error<T> {
		Uninitilized,
		AlreadyInitialized,
		ZeroSupplyProvided,
		InsufficientDataProvided,
		InsufficientFunds,
		TransferNotApproved,
		Overflow,
		SelfTransfer,
		ZeroAmountTransfer,
		ZeroAdressTransfer,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		#[transactional]
		pub fn init(
			origin: OriginFor<T>,
			token_ids: Vec<T::TokenId>,
			initial_supplies: Vec<T::Balance>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			<Self as Erc1155<_>>::init(&sender, &token_ids, &initial_supplies)?;
			Self::deposit_event(Event::Initialized(sender, token_ids, initial_supplies));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn transfer_from_single(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			token_id: T::TokenId,
			amount: T::Balance,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			<Self as Erc1155<_>>::transfer_from_single(&from, &to, &token_id, &amount)?;
			Self::deposit_event(Event::TransferSingle(from, to, token_id, amount));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn transfer_from_batch(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			token_ids: Vec<T::TokenId>,
			amounts: Vec<T::Balance>,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			<Self as Erc1155<_>>::transfer_from_batch(&from, &to, &token_ids, &amounts)?;
			Self::deposit_event(Event::TransferBatch(from, to, token_ids, amounts));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn set_approval_for_all(
			origin: OriginFor<T>,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			<Self as Erc1155<_>>::set_approval_for_all(&sender, &operator, approved);
			Self::deposit_event(Event::ApprovalForAll(sender, operator, approved));
			Ok(())
		}
	}

	impl<T: Config> Erc1155<T::AccountId> for Pallet<T> {
		type TokenId = T::TokenId;
		type Balance = T::Balance;

		fn init(
			who: &T::AccountId,
			token_ids: &Vec<Self::TokenId>,
			initial_supplies: &Vec<Self::Balance>,
		) -> DispatchResult {
			ensure!(
				token_ids.len() == initial_supplies.len(),
				Error::<T>::InsufficientDataProvided
			);
			for (token_id, supply) in token_ids.iter().zip(initial_supplies) {
				let real_supply = supply
					.checked_mul(
						&(10 as u128)
							.checked_pow(T::Decimals::get())
							.unwrap()
							.try_into()
							.ok()
							.unwrap(),
					)
					.ok_or(Error::<T>::Overflow)?;
				Self::token_uninitialized(token_id)?;
				ensure!(!real_supply.is_zero(), Error::<T>::ZeroSupplyProvided);
				Balances::<T>::insert(who, token_id, real_supply);
				TotalSupply::<T>::insert(token_id, real_supply);
			}
			Ok(())
		}

		fn total_supply(
			token_id: Self::TokenId,
		) -> Result<Self::Balance, sp_runtime::DispatchError> {
			Self::token_initialized(&token_id)?;
			Ok(Self::get_total_supply(token_id))
		}

		fn balance_of(
			account: &T::AccountId,
			token_id: Self::TokenId,
		) -> Result<Self::Balance, sp_runtime::DispatchError> {
			Self::token_initialized(&token_id)?;
			Ok(Self::get_balance(account, token_id))
		}

		fn balance_of_batch(
			accounts: Vec<&T::AccountId>,
			token_ids: Vec<Self::TokenId>,
		) -> Result<Vec<Self::Balance>, sp_runtime::DispatchError> {
			ensure!(accounts.len() == token_ids.len(), Error::<T>::InsufficientDataProvided);
			let mut balance = Vec::new();
			for (account, id) in accounts.iter().zip(token_ids) {
				balance.push(Self::balance_of(account, id).ok().unwrap());
			}
			Ok(balance)
		}

		fn transfer_from_single(
			from: &T::AccountId,
			to: &T::AccountId,
			token_id: &Self::TokenId,
			amount: &Self::Balance,
		) -> DispatchResult {
			Self::token_initialized(&token_id)?;
			ensure!(!amount.is_zero(), Error::<T>::ZeroAmountTransfer);
			ensure!(to != &T::AccountId::default(), Error::<T>::ZeroAdressTransfer);
			ensure!(from != to, Error::<T>::SelfTransfer);
			ensure!(Self::get_approval(from, to), Error::<T>::TransferNotApproved);
			Self::_transfer(from, to, token_id, amount)
		}

		fn transfer_from_batch(
			from: &T::AccountId,
			to: &T::AccountId,
			token_ids: &Vec<Self::TokenId>,
			amounts: &Vec<Self::Balance>,
		) -> DispatchResult {
			ensure!(token_ids.len() == amounts.len(), Error::<T>::InsufficientDataProvided);
			ensure!(to != &T::AccountId::default(), Error::<T>::ZeroAdressTransfer);
			ensure!(from != to, <Error<T>>::SelfTransfer);
			ensure!(Self::get_approval(from, to), Error::<T>::TransferNotApproved);
			for (token_id, amount) in token_ids.iter().zip(amounts) {
				Self::token_initialized(&token_id)?;
				ensure!(!amount.is_zero(), Error::<T>::ZeroAmountTransfer);
				Self::_transfer(from, to, token_id, amount)?;
			}
			Ok(())
		}

		fn transfer(
			from: &T::AccountId,
			to: &T::AccountId,
			token_id: Self::TokenId,
			amount: Self::Balance,
		) -> DispatchResult {
			Self::token_initialized(&token_id)?;
			ensure!(!amount.is_zero(), Error::<T>::ZeroAmountTransfer);
			ensure!(to != &T::AccountId::default(), Error::<T>::ZeroAdressTransfer);
			ensure!(from != to, Error::<T>::SelfTransfer);
			Self::_transfer(from, to, &token_id, &amount)
		}

		fn set_approval_for_all(owner: &T::AccountId, operator: &T::AccountId, approved: bool) {
			Approval::<T>::insert(owner, operator, approved);
		}
	}

	impl<T: Config> Pallet<T> {
		fn _transfer(
			from: &T::AccountId,
			to: &T::AccountId,
			token_id: &T::TokenId,
			amount: &T::Balance,
		) -> DispatchResult {
			Balances::<T>::try_mutate(&from, &token_id, |balance| -> Result<(), Error<T>> {
				let updated_sender_balance =
					balance.checked_sub(&amount).ok_or(Error::<T>::InsufficientFunds)?;
				*balance = updated_sender_balance;
				Ok(())
			})?;
			Balances::<T>::try_mutate(&to, &token_id, |balance| -> Result<(), Error<T>> {
				let updated_to_balance =
					balance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*balance = updated_to_balance;
				Ok(())
			})?;
			Ok(())
		}

		fn token_initialized(token_id: &T::TokenId) -> DispatchResult {
			ensure!(Self::is_initialized(token_id), Error::<T>::Uninitilized);
			Ok(())
		}

		fn token_uninitialized(token_id: &T::TokenId) -> DispatchResult {
			ensure!(!Self::is_initialized(token_id), Error::<T>::AlreadyInitialized);
			Ok(())
		}

		fn is_initialized(token_id: &T::TokenId) -> bool {
			!Self::get_total_supply(&token_id).is_zero()
		}
	}
}
