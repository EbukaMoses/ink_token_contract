#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod simple_token {
    use ink::storage::Mapping;

    #[derive(scale::Encode, scale::Decode, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Not enough balance to perform the operation
        InsufficientBalance,
        /// Caller is not the contract owner
        NotOwner,
        /// Operation is not allowed for the spender
        InsufficientAllowance,
        /// Contract is currently paused
        ContractPaused,
        /// Address is blacklisted
        AddressBlacklisted,
        /// Invalid amount (e.g., zero amount)
        InvalidAmount,
    }

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

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// Storage for allowances: (owner, spender) -> amount
    type Allowance = (AccountId, AccountId);

    #[ink(storage)]
    pub struct SimpleToken {
        /// Mapping from account to balance
        balances: Mapping<AccountId, Balance>,
        /// Contract owner
        owner: AccountId,
        /// Total token supply
        total_supply: Balance,
        /// Allowances for token transfers
        allowances: Mapping<Allowance, Balance>,
        /// Whether the contract is paused
        paused: bool,
        /// Blacklisted addresses
        blacklist: Mapping<AccountId, bool>,
    }

    impl SimpleToken {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self {
                balances: Mapping::default(),
                owner: caller,
                total_supply: 0,
                allowances: Mapping::default(),
                paused: false,
                blacklist: Mapping::default(),
            }
        }

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

        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> Balance {
            self.balances.get(&account).unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: Balance) -> Result<(), Error> {
            if self.paused {
                return Err(Error::ContractPaused);
            }
            
            let caller = self.env().caller();
            
            if self.blacklist.get(&caller).unwrap_or(false) || self.blacklist.get(&to).unwrap_or(false) {
                return Err(Error::AddressBlacklisted);
            }
            
            if amount == 0 {
                return Err(Error::InvalidAmount);
            }
            
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

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        
        /// Approve a spender to transfer tokens on behalf of the caller
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, amount: Balance) -> Result<(), Error> {
            let owner = self.env().caller();
            self.allowances.insert(&(owner, spender), &amount);
            
            self.env().emit_event(Approval {
                owner,
                spender,
                value: amount,
            });
            
            Ok(())
        }
        
        /// Get the allowance of a spender for an owner
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).unwrap_or(0)
        }
        
        /// Transfer tokens from one account to another using allowance
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
        ) -> Result<(), Error> {
            if self.paused {
                return Err(Error::ContractPaused);
            }
            
            if self.blacklist.get(&from).unwrap_or(false) || self.blacklist.get(&to).unwrap_or(false) {
                return Err(Error::AddressBlacklisted);
            }
            
            if amount == 0 {
                return Err(Error::InvalidAmount);
            }
            
            let caller = self.env().caller();
            let allowance = self.allowances.get(&(from, caller)).unwrap_or(0);
            
            if allowance < amount {
                return Err(Error::InsufficientAllowance);
            }
            
            let from_balance = self.balances.get(&from).unwrap_or(0);
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }
            
            // Update the allowance
            self.allowances.insert(&(from, caller), &(allowance - amount));
            
            // Update balances
            self.balances.insert(&from, &(from_balance - amount));
            let to_balance = self.balances.get(&to).unwrap_or(0);
            self.balances.insert(&to, &(to_balance + amount));
            
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value: amount,
            });
            
            Ok(())
        }
        
        /// Pause all token transfers (only owner)
        #[ink(message)]
        pub fn pause(&mut self) -> Result<(), Error> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.paused = true;
            Ok(())
        }

        /// Unpause token transfers (only owner)
        #[ink(message)]
        pub fn unpause(&mut self) -> Result<(), Error> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.paused = false;
            Ok(())
        }

        /// Check if the contract is paused
        #[ink(message)]
        pub fn is_paused(&self) -> bool {
            self.paused
        }

        /// Add an address to the blacklist (only owner)
        #[ink(message)]
        pub fn add_to_blacklist(&mut self, address: AccountId) -> Result<(), Error> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.blacklist.insert(&address, &true);
            Ok(())
        }

        /// Remove an address from the blacklist (only owner)
        #[ink(message)]
        pub fn remove_from_blacklist(&mut self, address: AccountId) -> Result<(), Error> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.blacklist.remove(&address);
            Ok(())
        }

        /// Check if an address is blacklisted
        #[ink(message)]
        pub fn is_blacklisted(&self, address: AccountId) -> bool {
            self.blacklist.get(&address).unwrap_or(false)
        }

        /// Transfer tokens to multiple addresses in a single transaction
        /// Each recipient will receive the same amount
        #[ink(message)]
        pub fn batch_transfer(
            &mut self,
            recipients: Vec<AccountId>,
            amount: Balance,
        ) -> Result<(), Error> {
            if self.paused {
                return Err(Error::ContractPaused);
            }
            
            let caller = self.env().caller();
            
            // Check if caller is blacklisted
            if self.blacklist.get(&caller).unwrap_or(false) {
                return Err(Error::AddressBlacklisted);
            }
            
            // Check if any recipient is blacklisted
            for recipient in &recipients {
                if self.blacklist.get(recipient).unwrap_or(false) {
                    return Err(Error::AddressBlacklisted);
                }
            }
            
            if amount == 0 {
                return Err(Error::InvalidAmount);
            }
            
            let total_amount = amount.checked_mul(recipients.len() as u128)
                .ok_or(Error::InsufficientBalance)?;
                
            let caller_balance = self.balances.get(&caller).unwrap_or(0);
            if caller_balance < total_amount {
                return Err(Error::InsufficientBalance);
            }
            
            // Update sender's balance
            self.balances.insert(&caller, &(caller_balance - total_amount));
            
            // Update recipients' balances
            for recipient in recipients {
                let recipient_balance = self.balances.get(&recipient).unwrap_or(0);
                self.balances.insert(&recipient, &(recipient_balance + amount));
                
                // Emit transfer event for each recipient
                self.env().emit_event(Transfer {
                    from: Some(caller),
                    to: Some(recipient),
                    value: amount,
                });
            }
            
            Ok(())
        }

        /// Burn tokens from the caller's account
        #[ink(message)]
        pub fn burn(&mut self, amount: Balance) -> Result<(), Error> {
            if amount == 0 {
                return Err(Error::InvalidAmount);
            }
            
            let caller = self.env().caller();
            let current_balance = self.balances.get(&caller).unwrap_or(0);
            
            if current_balance < amount {
                return Err(Error::InsufficientBalance);
            }
            
            self.balances.insert(&caller, &(current_balance - amount));
            self.total_supply -= amount;
            
            self.env().emit_event(Transfer {
                from: Some(caller),
                to: None,
                value: amount,
            });
            
            Ok(())
        }
    }
}
