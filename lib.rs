#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod traits;

#[ink::contract]
mod psp_coin {
    use ink::{storage::Mapping, H160, U256};

    use crate::{
        traits::{PSP22Burnable, PSP22Metadata, PSP22Mintable, PSP22},
    };

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct PspCoin {
        total_supply: U256,
        balances: Mapping<H160, U256>,
        // can owner authorize (allowance > balance)?
        allowances: Mapping<(H160, H160), U256>, // (owner, spender) -> allowance
        metadata: (String, String, u8)
    }

    impl PspCoin {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            todo!()
        }

        #[ink(constructor)]
        pub fn new_with_supply(total_supply: U256) -> Self {
            todo!()
        }
    }

    impl PSP22 for PspCoin {
        #[ink(message)]
        fn total_supply(&self) -> U256 {
            self.total_supply
        }
    }

    impl PSP22Metadata for PspCoin {}

    impl PSP22Mintable for PspCoin {}

    impl PSP22Burnable for PspCoin {}
}
