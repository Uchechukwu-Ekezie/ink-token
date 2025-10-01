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
        /// Account that sent the tokens
        #[ink(topic)]
        from: AccountId,
        /// Account that received the tokens
        #[ink(topic)]
        to: AccountId,
        /// Amount of tokens transferred
        amount: u128,
    }

    /// The Token contract storage
    /// Like a simple bank that keeps track of everyone's money
    #[ink(storage)]
    pub struct Token {
        /// Mapping from account to balance (like a phone book: person -> amount of money)
        balances: Mapping<AccountId, u128>,
        /// The contract owner who can mint new tokens
        owner: AccountId,
        /// Total supply of tokens
        total_supply: u128,
    }

    impl Default for Token {
        fn default() -> Self {
            Self::new()
        }
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
            let current_balance = self.balances.get(to).unwrap_or(0);

            // Add the new tokens to their balance
            let new_balance = current_balance.checked_add(amount)
                .expect("Overflow when adding to balance");
            self.balances.insert(to, &new_balance);

            // Update total supply
            self.total_supply = self.total_supply.checked_add(amount)
                .expect("Overflow when updating total supply");

            // Emit a Mint event (like sending a receipt)
            self.env().emit_event(Mint { to, amount });

            Ok(())
        }

        /// Check how much money someone has (like checking your bank balance)
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            self.balances.get(account).unwrap_or(0)
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
            let from_balance = self.balances.get(from).unwrap_or(0);

            // Check if sender has enough tokens
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            // Get the receiver's current balance
            let to_balance = self.balances.get(to).unwrap_or(0);

            // Update balances
            let new_from_balance = from_balance.checked_sub(amount)
                .expect("Underflow when subtracting from balance");
            let new_to_balance = to_balance.checked_add(amount)
                .expect("Overflow when adding to balance");

            self.balances.insert(from, &new_from_balance);
            self.balances.insert(to, &new_to_balance);

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

    /// Unit tests for the token contract
    #[cfg(test)]
    mod tests {
        use super::*;

        /// Test that the contract is properly initialized
        #[ink::test]
        fn new_works() {
            let token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            // The owner should be the account that deployed the contract
            assert_eq!(token.owner(), accounts.alice);
            assert_eq!(token.total_supply(), 0);
            assert_eq!(token.balance_of(accounts.alice), 0);
        }

        /// Test minting tokens
        #[ink::test]
        fn mint_works() {
            let mut token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Mint 100 tokens to Alice
            assert_eq!(token.mint(accounts.alice, 100), Ok(()));
            assert_eq!(token.balance_of(accounts.alice), 100);
            assert_eq!(token.total_supply(), 100);
        }

        /// Test that only owner can mint
        #[ink::test]
        fn mint_unauthorized_fails() {
            let mut token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Set caller to Bob (not the owner)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            // Bob tries to mint - should fail
            assert_eq!(token.mint(accounts.alice, 100), Err(Error::Unauthorized));
            assert_eq!(token.balance_of(accounts.alice), 0);
            assert_eq!(token.total_supply(), 0);
        }

        /// Test successful token transfer
        #[ink::test]
        fn transfer_works() {
            let mut token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // First, mint some tokens to Alice
            assert_eq!(token.mint(accounts.alice, 100), Ok(()));

            // Set caller to Alice
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            // Alice transfers 30 tokens to Bob
            assert_eq!(token.transfer(accounts.bob, 30), Ok(()));
            assert_eq!(token.balance_of(accounts.alice), 70);
            assert_eq!(token.balance_of(accounts.bob), 30);
        }

        /// Test transfer with insufficient balance
        #[ink::test]
        fn transfer_insufficient_balance_fails() {
            let mut token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Alice has no tokens, tries to transfer 10
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            assert_eq!(token.transfer(accounts.bob, 10), Err(Error::InsufficientBalance));
        }

        /// Test transfer to self fails
        #[ink::test]
        fn transfer_to_self_fails() {
            let mut token = Token::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Mint tokens to Alice
            assert_eq!(token.mint(accounts.alice, 100), Ok(()));

            // Set caller to Alice
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            // Alice tries to transfer to herself
            assert_eq!(token.transfer(accounts.alice, 10), Err(Error::SelfTransfer));
        }
    }

    /// End-to-end tests
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// Test that we can deploy and mint tokens
        #[ink_e2e::test]
        async fn e2e_mint_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Deploy the contract
            let mut constructor = TokenRef::new();
            let contract = client
                .instantiate("token", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Token>();

            // Mint 100 tokens to Alice
            let mint = call_builder.mint(ink_e2e::account_id(ink_e2e::AccountKeyring::Alice), 100);
            let _mint_result = client
                .call(&ink_e2e::alice(), &mint)
                .submit()
                .await
                .expect("mint failed");

            // Check Alice's balance
            let balance_of = call_builder.balance_of(ink_e2e::account_id(ink_e2e::AccountKeyring::Alice));
            let balance_result = client.call(&ink_e2e::alice(), &balance_of).dry_run().await?;
            assert_eq!(balance_result.return_value(), 100);

            // Check total supply
            let total_supply = call_builder.total_supply();
            let supply_result = client.call(&ink_e2e::alice(), &total_supply).dry_run().await?;
            assert_eq!(supply_result.return_value(), 100);

            Ok(())
        }

        /// Test token transfer
        #[ink_e2e::test]
        async fn e2e_transfer_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Deploy the contract
            let mut constructor = TokenRef::new();
            let contract = client
                .instantiate("token", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Token>();

            // Mint 100 tokens to Alice
            let mint = call_builder.mint(ink_e2e::account_id(ink_e2e::AccountKeyring::Alice), 100);
            let _mint_result = client
                .call(&ink_e2e::alice(), &mint)
                .submit()
                .await
                .expect("mint failed");

            // Alice transfers 30 tokens to Bob
            let transfer = call_builder.transfer(ink_e2e::account_id(ink_e2e::AccountKeyring::Bob), 30);
            let _transfer_result = client
                .call(&ink_e2e::alice(), &transfer)
                .submit()
                .await
                .expect("transfer failed");

            // Check balances
            let alice_balance = call_builder.balance_of(ink_e2e::account_id(ink_e2e::AccountKeyring::Alice));
            let alice_result = client.call(&ink_e2e::alice(), &alice_balance).dry_run().await?;
            assert_eq!(alice_result.return_value(), 70);

            let bob_balance = call_builder.balance_of(ink_e2e::account_id(ink_e2e::AccountKeyring::Bob));
            let bob_result = client.call(&ink_e2e::alice(), &bob_balance).dry_run().await?;
            assert_eq!(bob_result.return_value(), 30);

            Ok(())
        }
    }
}
