use codec::FullCodec;

use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
	vec::Vec,
};

pub trait Erc1155<AccountId> {
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

	fn init(
		who: &AccountId,
		token_ids: &Vec<Self::TokenId>,
		initial_supplies: &Vec<Self::Balance>,
	) -> DispatchResult;

	fn total_supply(token_id: Self::TokenId) -> Result<Self::Balance, sp_runtime::DispatchError>;

	fn balance_of(
		account: &AccountId,
		token_id: Self::TokenId,
	) -> Result<Self::Balance, sp_runtime::DispatchError>;

	fn balance_of_batch(
		accounts: Vec<&AccountId>,
		token_ids: Vec<Self::TokenId>,
	) -> Result<Vec<Self::Balance>, sp_runtime::DispatchError>;

	fn transfer_from_single(
		from: &AccountId,
		to: &AccountId,
		token_id: &Self::TokenId,
		amount: &Self::Balance,
	) -> DispatchResult;

	fn transfer_from_batch(
		from: &AccountId,
		to: &AccountId,
		token_ids: &Vec<Self::TokenId>,
		amounts: &Vec<Self::Balance>,
	) -> DispatchResult;

	fn transfer(
		from: &AccountId,
		to: &AccountId,
		token_id: Self::TokenId,
		amount: Self::Balance,
	) -> DispatchResult;
	
	fn set_approval_for_all(owner: &AccountId, operator: &AccountId, approved: bool);
}
