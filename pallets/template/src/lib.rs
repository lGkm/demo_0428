#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

/// 开始定义功能模块
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, storage::bounded_vec::BoundedVec};
	use frame_system::pallet_prelude::*;


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


	/// 模块配置接口，继承自系统模块
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type MaxBytesInHash: Get<u32>;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a proof has been claimed. [who, claim]
		ClaimCreated(T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
		/// Event emitted when a claim is revoked by the owner. [who, claim]
		ClaimRevoked(T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),

		ClaimTransaction(T::AccountId, T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The proof has already been claimed.
		ProofAlreadyClaimed,
		/// The proof does not exist, so it cannot be revoked.
		NoSuchProof,
		/// The proof is claimed by another account, so caller can't revoke it.
		NotProofOwner,
	}

	//可调用函数
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn create_claim(
			origin:OriginFor<T>,
			proof:BoundedVec<u8,T::MaxBytesInHash>,
		) -> DispatchResult {
			// 检查是否签名
			let sender = ensure_signed(origin)?;
			//检查 proof 是否已经被 claimed， 是的话 报错 ProofAlreadyClaimed
			ensure!(!Proofs::<T>::contains_key(&proof),Error::<T>::ProofAlreadyClaimed);
			// 获得当前的块号
			let current_block = <frame_system::Pallet<T>>::block_number();
			// 向 StorageMap 中插入
			Proofs::<T>::insert(&proof, (&sender, current_block));
			// 发送 ClaimCreated 通知
			Self::deposit_event(Event::ClaimCreated(sender, proof));
			Ok(())
		}
		#[pallet::weight(1_000)]
		pub fn revole_claim(
			origin:OriginFor<T>,
			proof:BoundedVec<u8,T::MaxBytesInHash>
		)->DispatchResult{
			let sender = ensure_signed(origin)?;
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);
			// StorageMap 获得 value
			let (owner, _) = Proofs::<T>::get(&proof).expect("All proofs must have an owner!");

			ensure!(sender == owner, Error::<T>::NotProofOwner);

			Proofs::<T>::remove(&proof);

			Self::deposit_event(Event::ClaimRevoked(sender, proof));
			Ok(())
		}
		#[pallet::weight(1_000)]
		pub fn transaction_claim(
			origin:OriginFor<T>,
			proof:BoundedVec<u8,T::MaxBytesInHash>,
			to_address:T::AccountId
		)->DispatchResult{
			let sender = ensure_signed(origin)?;
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);
			// StorageMap 获得 value
			let (owner, _) = Proofs::<T>::get(&proof).expect("All proofs must have an owner!");

			ensure!(sender == owner, Error::<T>::NotProofOwner);
			// 获得当前的块号
			let current_block = <frame_system::Pallet<T>>::block_number();
			Proofs::<T>::remove(&proof);
			// 向 StorageMap 中插入
			Proofs::<T>::insert(&proof, (&to_address, current_block));

			Self::deposit_event(Event::ClaimTransaction(sender, to_address,proof));
			Ok(())
		}

	}

	////定义存证单元 可选择get函数
	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub(super) type Proofs<T:Config> = StorageMap<
		_,
		Blake2_128Concat,//key加密方式
		BoundedVec<u8,T::MaxBytesInHash>,//key
		(T::AccountId,T::BlockNumber),//value
		OptionQuery,//查询方式
	>;
}