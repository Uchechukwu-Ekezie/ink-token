#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod token {
    use ink::storage::Mapping;

    /// Custom error types for the token contract
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Insufficient balance for the operation
        InsufficientBalance,
        /// Unauthorized access - only owner can perform this operation
        Unauthorized,
        /// Cannot transfer to the same account
        SelfTransfer,
    }

    /// Result type for contract operations
    pub type Result<T> = core::result::Result<T, Error>;

    /// Events emitted by the token contract
    #[ink(event)]
    pub struct Mint {
        /// Account that received the minted tokens
        #[ink(topic)]
        to: AccountId,
        /// Amount of tokens minted
        amount: u128,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,

        #[ink(topic)]
        to: AccountId,
        /// Amount of tokens transferred
        amount: u128,
    }

    #[ink(storage)]
    pub struct Token {
        /// Mapping from account to balance (like a phone book: person -> amount of money)
        balances: Mapping<AccountId, u128>,
        /// The contract owner who can mint new tokens
        owner: AccountId,
        /// Total supply of tokens
        total_supply: u128,
    }

    impl Token {
        /// Constructor that creates a new token contract
        /// The person who deploys the contract becomes the owner
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self {
                balances: Mapping::default(),
                owner: caller,
                total_supply: 0,
            }
        }

        /// Mint new tokens (like printing new money)
        /// Only the owner can call this function
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, amount: u128) -> Result<()> {
            // Check if the caller is the owner
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            // Get the current balance of the account (default to 0 if not found)
            let current_balance = self.balances.get(&to).unwrap_or(0);

            // Add the new tokens to their balance
            let new_balance = current_balance + amount;
            self.balances.insert(&to, &new_balance);

            // Update total supply
            self.total_supply += amount;

            // Emit a Mint event (like sending a receipt)
            self.env().emit_event(Mint { to, amount });

            Ok(())
        }

        /// Check how much money someone has (like checking your bank balance)
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            self.balances.get(&account).unwrap_or(0)
        }

        /// Transfer tokens from one account to another (like sending money to someone)
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let from = self.env().caller();

            // Prevent self-transfer
            if from == to {
                return Err(Error::SelfTransfer);
            }

            // Get the sender's current balance
            let from_balance = self.balances.get(&from).unwrap_or(0);

            // Check if sender has enough tokens
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            // Get the receiver's current balance
            let to_balance = self.balances.get(&to).unwrap_or(0);

            // Update balances
            self.balances.insert(&from, &(from_balance - amount));
            self.balances.insert(&to, &(to_balance + amount));

            // Emit a Transfer event (like sending a receipt)
            self.env().emit_event(Transfer { from, to, amount });

            Ok(())
        }

        /// Get the contract owner
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Get the total supply of tokens
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }
    }
}/// Unit tests for the token contract
