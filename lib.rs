#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod traits;

#[ink::contract]
mod psp_coin {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::{H160, U256, storage::Mapping};

    use crate::data::PSP22Error;

    /// Event emitted when tokens are transferred
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<H160>,
        #[ink(topic)]
        to: Option<H160>,
        value: U256,
    }

    /// Event emitted when approval is granted
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: H160,
        #[ink(topic)]
        spender: H160,
        value: U256,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct PspCoin {
        total_supply: U256,
        balances: Mapping<H160, U256>,
        // can owner authorize (allowance > balance)?
        allowances: Mapping<(H160, H160), U256>, // (owner, spender) -> allowance
        metadata: (String, String, u8),
    }

    impl PspCoin {
        /// Constructor that initializes a memecoin with zero supply
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                total_supply: U256::from(0),
                balances: Mapping::default(),
                allowances: Mapping::default(),
                metadata: (String::from("MemeCoin"), String::from("MEME"), 18),
            }
        }

        /// Constructor that initializes a memecoin with initial supply
        #[ink(constructor)]
        pub fn new_with_supply(total_supply: U256) -> Self {
            let caller_h160 = Self::env().caller();

            let mut balances = Mapping::default();
            balances.insert(caller_h160, &total_supply);

            Self {
                total_supply,
                balances,
                allowances: Mapping::default(),
                metadata: (String::from("MemeCoin"), String::from("MEME"), 18),
            }
        }

        /// Helper function to get the caller as H160
        fn caller(&self) -> H160 {
            self.env().caller()
        }

        /// Internal transfer function
        fn transfer_from_to(
            &mut self,
            from: H160,
            to: H160,
            value: U256,
        ) -> Result<(), PSP22Error> {
            // No-op if from and to are the same or value is zero
            if from == to || value.is_zero() {
                return Ok(());
            }

            let from_balance = self.balances.get(from).unwrap_or(U256::from(0));

            if from_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            let to_balance = self.balances.get(to).unwrap_or(U256::from(0));

            // Check for overflow
            if to_balance.checked_add(value).is_none() {
                return Err(PSP22Error::Overflow);
            }

            self.balances.insert(from, &(from_balance - value));
            self.balances.insert(to, &(to_balance + value));

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            Ok(())
        }
    }

    impl PspCoin {
        // PSP22 Standard Functions

        /// Returns the total token supply
        #[ink(message)]
        pub fn total_supply(&self) -> U256 {
            self.total_supply
        }

        /// Returns the balance of an account
        #[ink(message)]
        pub fn balance_of(&self, owner: H160) -> U256 {
            self.balances.get(owner).unwrap_or(U256::from(0))
        }

        /// Returns the allowance of a spender for an owner
        #[ink(message)]
        pub fn allowance(&self, owner: H160, spender: H160) -> U256 {
            self.allowances
                .get((owner, spender))
                .unwrap_or(U256::from(0))
        }

        /// Transfers tokens from the caller to another account
        #[ink(message)]
        pub fn transfer(
            &mut self,
            to: H160,
            value: U256,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let from = self.caller();
            self.transfer_from_to(from, to, value)
        }

        /// Transfers tokens from one account to another using allowance
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: H160,
            to: H160,
            value: U256,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let caller = self.caller();

            // No-op if from and to are the same or value is zero
            if from == to || value.is_zero() {
                return Ok(());
            }

            // If caller is not the owner, check allowance
            if caller != from {
                let allowance = self.allowances.get((from, caller)).unwrap_or(U256::from(0));

                if allowance < value {
                    return Err(PSP22Error::InsufficientAllowance);
                }

                // Decrease allowance
                self.allowances.insert((from, caller), &(allowance - value));

                self.env().emit_event(Approval {
                    owner: from,
                    spender: caller,
                    value: allowance - value,
                });
            }

            self.transfer_from_to(from, to, value)
        }

        /// Approves a spender to spend tokens on behalf of the caller
        #[ink(message)]
        pub fn approve(&mut self, spender: H160, value: U256) -> Result<(), PSP22Error> {
            let owner = self.caller();

            // No-op if owner and spender are the same
            if owner == spender {
                return Ok(());
            }

            self.allowances.insert((owner, spender), &value);

            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });

            Ok(())
        }

        /// Increases the allowance of a spender
        #[ink(message)]
        pub fn increase_allowance(
            &mut self,
            spender: H160,
            delta_value: U256,
        ) -> Result<(), PSP22Error> {
            let owner = self.caller();

            // No-op if owner and spender are the same or delta_value is zero
            if owner == spender || delta_value.is_zero() {
                return Ok(());
            }

            let current_allowance = self
                .allowances
                .get((owner, spender))
                .unwrap_or(U256::from(0));
            let new_allowance = current_allowance
                .checked_add(delta_value)
                .ok_or(PSP22Error::Overflow)?;

            self.allowances.insert((owner, spender), &new_allowance);

            self.env().emit_event(Approval {
                owner,
                spender,
                value: new_allowance,
            });

            Ok(())
        }

        /// Decreases the allowance of a spender
        #[ink(message)]
        pub fn decrease_allowance(
            &mut self,
            spender: H160,
            delta_value: U256,
        ) -> Result<(), PSP22Error> {
            let owner = self.caller();

            // No-op if owner and spender are the same or delta_value is zero
            if owner == spender || delta_value.is_zero() {
                return Ok(());
            }

            let current_allowance = self
                .allowances
                .get((owner, spender))
                .unwrap_or(U256::from(0));

            if current_allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);
            }

            let new_allowance = current_allowance - delta_value;
            self.allowances.insert((owner, spender), &new_allowance);

            self.env().emit_event(Approval {
                owner,
                spender,
                value: new_allowance,
            });

            Ok(())
        }

        // PSP22 Metadata Functions

        /// Returns the token name
        #[ink(message)]
        pub fn name(&self) -> Option<String> {
            Some(self.metadata.0.clone())
        }

        /// Returns the token symbol
        #[ink(message)]
        pub fn symbol(&self) -> Option<String> {
            Some(self.metadata.1.clone())
        }

        /// Returns the token decimals
        #[ink(message)]
        pub fn decimals(&self) -> u8 {
            self.metadata.2
        }

        // PSP22 Mintable Functions

        /// Mints new tokens to the caller's account
        #[ink(message)]
        pub fn mint(&mut self, value: U256) -> Result<(), PSP22Error> {
            // No-op if value is zero
            if value.is_zero() {
                return Ok(());
            }

            let caller = self.caller();
            let balance = self.balances.get(caller).unwrap_or(U256::from(0));

            // Check for overflow
            let new_balance = balance.checked_add(value).ok_or(PSP22Error::Overflow)?;
            let new_supply = self
                .total_supply
                .checked_add(value)
                .ok_or(PSP22Error::Overflow)?;

            self.balances.insert(caller, &new_balance);
            self.total_supply = new_supply;

            self.env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value,
            });

            Ok(())
        }

        // PSP22 Burnable Functions

        /// Burns tokens from the caller's account
        #[ink(message)]
        pub fn burn(&mut self, value: U256) -> Result<(), PSP22Error> {
            // No-op if value is zero
            if value.is_zero() {
                return Ok(());
            }

            let caller = self.caller();
            let balance = self.balances.get(caller).unwrap_or(U256::from(0));

            if balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            self.balances.insert(caller, &(balance - value));
            self.total_supply = self.total_supply - value;

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: None,
                value,
            });

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        fn default_accounts() -> test::DefaultAccounts {
            test::default_accounts()
        }

        fn set_caller(caller: H160) {
            test::set_caller(caller);
        }

        #[ink::test]
        fn new_works() {
            let token = PspCoin::new();
            assert_eq!(token.total_supply(), U256::from(0));
            assert_eq!(token.name(), Some(String::from("MemeCoin")));
            assert_eq!(token.symbol(), Some(String::from("MEME")));
            assert_eq!(token.decimals(), 18);
        }

        #[ink::test]
        fn new_with_supply_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let initial_supply = U256::from(1000000);
            let token = PspCoin::new_with_supply(initial_supply);

            assert_eq!(token.total_supply(), initial_supply);
            assert_eq!(token.balance_of(accounts.alice), initial_supply);
            assert_eq!(token.balance_of(accounts.bob), U256::from(0));
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let initial_supply = U256::from(1000);
            let mut token = PspCoin::new_with_supply(initial_supply);

            let transfer_amount = U256::from(100);
            assert!(
                token
                    .transfer(accounts.bob, transfer_amount, Vec::new())
                    .is_ok()
            );

            assert_eq!(token.balance_of(accounts.alice), U256::from(900));
            assert_eq!(token.balance_of(accounts.bob), U256::from(100));
        }

        #[ink::test]
        fn transfer_insufficient_balance_fails() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let initial_supply = U256::from(100);
            let mut token = PspCoin::new_with_supply(initial_supply);

            let transfer_amount = U256::from(200);
            assert_eq!(
                token.transfer(accounts.bob, transfer_amount, Vec::new()),
                Err(PSP22Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn transfer_to_self_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let initial_supply = U256::from(1000);
            let mut token = PspCoin::new_with_supply(initial_supply);

            assert!(
                token
                    .transfer(accounts.alice, U256::from(100), Vec::new())
                    .is_ok()
            );
            assert_eq!(token.balance_of(accounts.alice), initial_supply);
        }

        #[ink::test]
        fn approve_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            let allowance_amount = U256::from(200);
            assert!(token.approve(accounts.bob, allowance_amount).is_ok());

            assert_eq!(
                token.allowance(accounts.alice, accounts.bob),
                allowance_amount
            );
        }

        #[ink::test]
        fn approve_self_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            assert!(token.approve(accounts.alice, U256::from(100)).is_ok());
            assert_eq!(
                token.allowance(accounts.alice, accounts.alice),
                U256::from(0)
            );
        }

        #[ink::test]
        fn transfer_from_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            // Alice approves Bob to spend 200 tokens
            assert!(token.approve(accounts.bob, U256::from(200)).is_ok());

            // Bob transfers 100 tokens from Alice to Charlie
            set_caller(accounts.bob);
            assert!(
                token
                    .transfer_from(
                        accounts.alice,
                        accounts.charlie,
                        U256::from(100),
                        Vec::new()
                    )
                    .is_ok()
            );

            assert_eq!(token.balance_of(accounts.alice), U256::from(900));
            assert_eq!(token.balance_of(accounts.charlie), U256::from(100));
            assert_eq!(
                token.allowance(accounts.alice, accounts.bob),
                U256::from(100)
            );
        }

        #[ink::test]
        fn transfer_from_insufficient_allowance_fails() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            // Alice approves Bob to spend 50 tokens
            assert!(token.approve(accounts.bob, U256::from(50)).is_ok());

            // Bob tries to transfer 100 tokens from Alice
            set_caller(accounts.bob);
            assert_eq!(
                token.transfer_from(
                    accounts.alice,
                    accounts.charlie,
                    U256::from(100),
                    Vec::new()
                ),
                Err(PSP22Error::InsufficientAllowance)
            );
        }

        #[ink::test]
        fn transfer_from_by_owner_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            // Alice transfers her own tokens without approval
            assert!(
                token
                    .transfer_from(accounts.alice, accounts.bob, U256::from(100), Vec::new())
                    .is_ok()
            );

            assert_eq!(token.balance_of(accounts.alice), U256::from(900));
            assert_eq!(token.balance_of(accounts.bob), U256::from(100));
        }

        #[ink::test]
        fn increase_allowance_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            assert!(token.approve(accounts.bob, U256::from(100)).is_ok());
            assert!(
                token
                    .increase_allowance(accounts.bob, U256::from(50))
                    .is_ok()
            );

            assert_eq!(
                token.allowance(accounts.alice, accounts.bob),
                U256::from(150)
            );
        }

        #[ink::test]
        fn decrease_allowance_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            assert!(token.approve(accounts.bob, U256::from(100)).is_ok());
            assert!(
                token
                    .decrease_allowance(accounts.bob, U256::from(50))
                    .is_ok()
            );

            assert_eq!(
                token.allowance(accounts.alice, accounts.bob),
                U256::from(50)
            );
        }

        #[ink::test]
        fn decrease_allowance_below_zero_fails() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            assert!(token.approve(accounts.bob, U256::from(50)).is_ok());
            assert_eq!(
                token.decrease_allowance(accounts.bob, U256::from(100)),
                Err(PSP22Error::InsufficientAllowance)
            );
        }

        #[ink::test]
        fn mint_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new();

            assert!(token.mint(U256::from(1000)).is_ok());
            assert_eq!(token.total_supply(), U256::from(1000));
            assert_eq!(token.balance_of(accounts.alice), U256::from(1000));

            assert!(token.mint(U256::from(500)).is_ok());
            assert_eq!(token.total_supply(), U256::from(1500));
            assert_eq!(token.balance_of(accounts.alice), U256::from(1500));
        }

        #[ink::test]
        fn mint_zero_is_noop() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new();

            assert!(token.mint(U256::from(0)).is_ok());
            assert_eq!(token.total_supply(), U256::from(0));
            assert_eq!(token.balance_of(accounts.alice), U256::from(0));
        }

        #[ink::test]
        fn burn_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            assert!(token.burn(U256::from(300)).is_ok());
            assert_eq!(token.total_supply(), U256::from(700));
            assert_eq!(token.balance_of(accounts.alice), U256::from(700));
        }

        #[ink::test]
        fn burn_insufficient_balance_fails() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(100));

            assert_eq!(
                token.burn(U256::from(200)),
                Err(PSP22Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn burn_zero_is_noop() {
            let accounts = default_accounts();
            set_caller(accounts.alice);

            let mut token = PspCoin::new_with_supply(U256::from(1000));

            assert!(token.burn(U256::from(0)).is_ok());
            assert_eq!(token.total_supply(), U256::from(1000));
        }

        #[ink::test]
        fn metadata_works() {
            let token = PspCoin::new();

            assert_eq!(token.name(), Some(String::from("MemeCoin")));
            assert_eq!(token.symbol(), Some(String::from("MEME")));
            assert_eq!(token.decimals(), 18);
        }
    }
}
