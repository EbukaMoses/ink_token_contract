#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod simple_token {
    use ink::storage::Mapping;

    /// Errors for mint and transfer
    #[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        NotOwner,
    }

    /// Events
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[ink(storage)]
    pub struct SimpleToken {
        balances: Mapping<AccountId, Balance>,
        owner: AccountId,
        total_supply: Balance,
    }

    impl SimpleToken {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self {
                balances: Mapping::default(),
                owner: caller,
                total_supply: 0,
            }
        }

        /// Mint tokens (only owner)
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, amount: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            let current_balance = self.balances.get(&to).unwrap_or(0);
            self.balances.insert(&to, &(current_balance + amount));
            self.total_supply += amount;

            self.env().emit_event(Mint { to, value: amount });
            self.env().emit_event(Transfer {
                from: None,
                to: Some(to),
                value: amount,
            });
            Ok(())
        }

        /// Check balance
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> Balance {
            self.balances.get(&account).unwrap_or(0)
        }

        /// Transfer tokens
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            let caller_balance = self.balances.get(&caller).unwrap_or(0);

            if caller_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(&caller, &(caller_balance - amount));

            let receiver_balance = self.balances.get(&to).unwrap_or(0);
            self.balances.insert(&to, &(receiver_balance + amount));

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(to),
                value: amount,
            });
            Ok(())
        }

        /// Total supply
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }
    }
}
