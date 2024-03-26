#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, IntegerSquareRoot, Zero},
		Perbill,
	};
	use traits::Erc1155;

	type BalanceOf<T> =
		<<T as Config>::Tokens as Erc1155<<T as frame_system::Config>::AccountId>>::Balance;

	type TokenIdOf<T> =
		<<T as Config>::Tokens as Erc1155<<T as frame_system::Config>::AccountId>>::TokenId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Tokens: Erc1155<Self::AccountId>;
		#[pallet::constant]
		type Fee: Get<Perbill>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_pool_address)]
	pub(super) type PoolAddress<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn get_token_ids)]
	pub(super) type TokenIds<T: Config> = StorageValue<_, (TokenIdOf<T>, TokenIdOf<T>)>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_liquidity)]
	pub(super) type TotalLiquidity<T: Config> = StorageValue<_, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_liquidity)]
	pub(super) type Liquidity<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Initialized(
			T::AccountId,
			T::AccountId,
			TokenIdOf<T>,
			BalanceOf<T>,
			TokenIdOf<T>,
			BalanceOf<T>,
		),
		TokenBought(T::AccountId, TokenIdOf<T>, BalanceOf<T>, TokenIdOf<T>, BalanceOf<T>),
		Deposited(T::AccountId, TokenIdOf<T>, BalanceOf<T>, TokenIdOf<T>, BalanceOf<T>),
		Withdrawed(T::AccountId, TokenIdOf<T>, BalanceOf<T>, TokenIdOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		Uninitilized,
		AlreadyInitialized,
		WrongInitialization,
		WrongTokenId,
		Overflow,
		WrongShareValue,
		NoLiquiudity,
		NoLiquiudityToWithdraw,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000)]
		#[transactional]
		pub fn init(
			origin: OriginFor<T>,
			pool_address: T::AccountId,
			first_token_id: TokenIdOf<T>,
			first_token_amount: BalanceOf<T>,
			second_token_id: TokenIdOf<T>,
			second_token_amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::uninitialized()?;
			ensure!(
				!first_token_amount.is_zero()
					&& !second_token_amount.is_zero()
					&& pool_address != T::AccountId::default(),
				Error::<T>::WrongInitialization
			);
			T::Tokens::transfer_from_batch(
				&sender,
				&pool_address,
				&vec![first_token_id, second_token_id],
				&vec![first_token_amount, second_token_amount],
			)?;
			let total_liquidity = first_token_amount.checked_add(&second_token_amount).unwrap();
			TotalLiquidity::<T>::put(total_liquidity);
			Liquidity::<T>::insert(&sender, total_liquidity);
			TokenIds::<T>::put((first_token_id, second_token_id));
			PoolAddress::<T>::put(&pool_address);
			Self::deposit_event(Event::Initialized(
				sender,
				pool_address,
				first_token_id,
				first_token_amount,
				second_token_id,
				second_token_amount,
			));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn buy_token(
			origin: OriginFor<T>,
			token_id: TokenIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::initialized()?;
			Self::has_liquidity()?;
			let token_to_buy = Self::get_paired_token(token_id).unwrap();
			let pool = Self::get_pool_address().unwrap();
			let reserves =
				T::Tokens::balance_of_batch(vec![&pool, &pool], vec![token_id, token_to_buy])?;
			let bought = Self::price(amount, reserves[0], reserves[1]).unwrap();
			T::Tokens::transfer_from_single(&sender, &pool, &token_id, &amount)?;
			T::Tokens::transfer(&pool, &sender, token_to_buy, bought)?;
			Self::deposit_event(Event::TokenBought(sender, token_id, amount, token_to_buy, bought));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn deposit(
			origin: OriginFor<T>,
			token_id: TokenIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::initialized()?;
			let paired_token = Self::get_paired_token(token_id).unwrap();
			let pool = Self::get_pool_address().unwrap();
			let reserves =
				T::Tokens::balance_of_batch(vec![&pool, &pool], vec![token_id, paired_token])?;
			let second_token_amount =
				amount.checked_mul(&reserves[1]).unwrap().checked_div(&reserves[0]).unwrap();

			Self::increase_liquidity(&sender, amount.checked_add(&second_token_amount).unwrap())?;
			T::Tokens::transfer_from_batch(
				&sender,
				&pool,
				&vec![token_id, paired_token],
				&vec![amount, second_token_amount],
			)?;

			Self::deposit_event(Event::Deposited(
				sender,
				token_id,
				amount,
				paired_token,
				second_token_amount,
			));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn deposit_single_token(
			origin: OriginFor<T>,
			token_id: TokenIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::initialized()?;
			let paired_token = Self::get_paired_token(token_id).unwrap();
			let pool = Self::get_pool_address().unwrap();
			let reserves =
				T::Tokens::balance_of_batch(vec![&pool, &pool], vec![token_id, paired_token])?;
			let (token_to_swap, bought_paired_token) =
				Self::calculate_single_token_ration(amount, reserves[0], reserves[1]).unwrap();

			Self::increase_liquidity(
				&sender,
				token_to_swap.checked_add(&bought_paired_token).unwrap(),
			)?;
			T::Tokens::transfer_from_single(&sender, &pool, &token_id, &amount)?;

			Self::deposit_event(Event::TokenBought(
				sender.clone(),
				token_id,
				token_to_swap,
				paired_token,
				bought_paired_token,
			));
			Self::deposit_event(Event::Deposited(
				sender,
				token_id,
				amount.checked_sub(&token_to_swap).unwrap(),
				paired_token,
				bought_paired_token,
			));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn withdraw(origin: OriginFor<T>, share_percent: u32) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::initialized()?;
			ensure!(share_percent > 0 && share_percent <= 100, Error::<T>::WrongShareValue);
			let share_percent = Perbill::from_percent(share_percent);

			let pool = Self::get_pool_address().unwrap();
			let (token_1, token_2) = Self::get_token_ids().unwrap();
			let reserves = T::Tokens::balance_of_batch(vec![&pool, &pool], vec![token_1, token_2])?;
			let total_liquidity = Self::get_total_liquidity().unwrap();

			let share_percent = share_percent
				* Perbill::from_rational(Self::get_liquidity(&sender), total_liquidity);
			let first_token_amount = share_percent * reserves[0];
			let second_token_amount = share_percent * reserves[1];

			Self::decrease_liquidity(&sender, share_percent * total_liquidity)?;
			T::Tokens::transfer(&pool, &sender, token_1, first_token_amount)?;
			T::Tokens::transfer(&pool, &sender, token_2, second_token_amount)?;
			Self::deposit_event(Event::Withdrawed(
				sender,
				token_1,
				first_token_amount,
				token_2,
				second_token_amount,
			));
			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn withdraw_single_token(
			origin: OriginFor<T>,
			token_id: TokenIdOf<T>,
			share_percent: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::initialized()?;
			ensure!(share_percent > 0 && share_percent <= 100, Error::<T>::WrongShareValue);
			let share_percent =
				Perbill::from_percent(share_percent) * Self::get_pool_share(&sender);
			ensure!(share_percent != Perbill::from_percent(0), Error::<T>::NoLiquiudityToWithdraw);

			let pool = Self::get_pool_address().unwrap();
			let paired_token = Self::get_paired_token(token_id).unwrap();
			let reserves =
				T::Tokens::balance_of_batch(vec![&pool, &pool], vec![token_id, paired_token])?;
			let total_liquidity = Self::get_total_liquidity().unwrap();

			let first_token_amount = share_percent * reserves[0];
			let second_token_amount = share_percent * reserves[1];

			let bought_first_token = Self::price(
				second_token_amount,
				reserves[1],
				reserves[0].checked_sub(&first_token_amount).unwrap(),
			)
			.unwrap();

			Self::decrease_liquidity(&sender, share_percent * total_liquidity)?;
			T::Tokens::transfer(
				&pool,
				&sender,
				token_id,
				first_token_amount.checked_add(&bought_first_token).unwrap(),
			)?;
			Self::deposit_event(Event::TokenBought(
				sender.clone(),
				paired_token,
				second_token_amount,
				token_id,
				bought_first_token,
			));
			Self::deposit_event(Event::Withdrawed(
				sender,
				token_id,
				first_token_amount,
				paired_token,
				BalanceOf::<T>::default(),
			));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn price(
			input_amount: BalanceOf<T>,
			input_reserve: BalanceOf<T>,
			output_reserve: BalanceOf<T>,
		) -> Option<BalanceOf<T>> {
			let input_amount_with_fee = T::Fee::get() * input_amount;
			input_amount_with_fee
				.checked_mul(&output_reserve)
				.unwrap()
				.checked_div(&input_reserve.checked_add(&input_amount_with_fee).unwrap())
		}

		/// Calculate the amount of input token we need to swap for second token to achieve correct ratio
		/// considering fee and the fact that token ration changed after we did token swap
		fn calculate_single_token_ration(
			input_amount: BalanceOf<T>,
			input_reserve: BalanceOf<T>,
			output_reserve: BalanceOf<T>,
		) -> Option<(BalanceOf<T>, BalanceOf<T>)> {
			let fee = T::Fee::get();
			let two: BalanceOf<T> = 2u32.try_into().ok().unwrap();
			let discriminant_sqrt = input_reserve
				.integer_sqrt_checked()
				.unwrap()
				.checked_mul(
					&((fee * fee * input_reserve)
						.checked_add(&(fee * input_reserve.checked_mul(&two).unwrap()))
						.unwrap()
						.checked_add(&(fee * two * two.checked_mul(&input_amount).unwrap()))
						.unwrap()
						.checked_add(&input_reserve)
						.unwrap()
						.integer_sqrt_checked()
						.unwrap()),
				)
				.unwrap();
			let tokens_to_swap = discriminant_sqrt
				.checked_sub(&input_reserve)
				.unwrap()
				.checked_sub(&(fee * input_reserve))
				.unwrap()
				.checked_div(&(fee * two))
				.unwrap();

			let bought = Self::price(tokens_to_swap, input_reserve, output_reserve).unwrap();
			Some((tokens_to_swap, bought))
		}

		fn get_paired_token(token_id: TokenIdOf<T>) -> Option<TokenIdOf<T>> {
			let (token_1, token_2) = Self::get_token_ids().unwrap();
			match token_id {
				t1 if t1 == token_1 => Some(token_2),
				t2 if t2 == token_2 => Some(token_1),
				_ => None,
			}
		}

		pub fn get_pool_share(owner: &T::AccountId) -> Perbill {
			Perbill::from_rational(
				Self::get_liquidity(&owner),
				Self::get_total_liquidity().unwrap(),
			)
		}

		pub fn get_reward(owner: &T::AccountId) -> BalanceOf<T> {
			Self::get_pool_share(owner) * Self::get_total_reward().unwrap()
		}

		pub fn get_total_reward() -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
			let pool = Self::get_pool_address().unwrap();
			let (token_1, token_2) = Self::get_token_ids().unwrap();
			let liquidity_with_fees =
				T::Tokens::balance_of(&pool, token_1)? + T::Tokens::balance_of(&pool, token_2)?;
			Ok(liquidity_with_fees.checked_sub(&Self::get_total_liquidity().unwrap()).unwrap())
		}

		fn initialized() -> Result<(), Error<T>> {
			ensure!(Self::is_initialized(), <Error<T>>::Uninitilized);
			Ok(())
		}

		fn uninitialized() -> Result<(), Error<T>> {
			ensure!(!Self::is_initialized(), <Error<T>>::AlreadyInitialized);
			Ok(())
		}

		fn has_liquidity() -> Result<(), Error<T>> {
			ensure!(
				Self::get_total_liquidity().unwrap() != BalanceOf::<T>::default(),
				<Error<T>>::NoLiquiudity
			);
			Ok(())
		}

		fn is_initialized() -> bool {
			Self::get_pool_address().is_some()
		}

		fn increase_liquidity(owner: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			TotalLiquidity::<T>::try_mutate(|liquidity| -> Result<(), Error<T>> {
				let updated_liquidity =
					liquidity.unwrap().checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*liquidity = Some(updated_liquidity);
				Ok(())
			})?;
			Liquidity::<T>::try_mutate(owner, |liquidity| -> Result<(), Error<T>> {
				let updated_liquidity =
					liquidity.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*liquidity = updated_liquidity;
				Ok(())
			})?;
			Ok(())
		}

		fn decrease_liquidity(owner: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			TotalLiquidity::<T>::try_mutate(|liquidity| -> Result<(), Error<T>> {
				let updated_liquidity =
					liquidity.unwrap().checked_sub(&amount).ok_or(Error::<T>::Overflow)?;
				*liquidity = Some(updated_liquidity);
				Ok(())
			})?;
			Liquidity::<T>::try_mutate(owner, |liquidity| -> Result<(), Error<T>> {
				let updated_liquidity =
					liquidity.checked_sub(&amount).ok_or(Error::<T>::Overflow)?;
				*liquidity = updated_liquidity;
				Ok(())
			})?;
			Ok(())
		}
	}
}
