#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod token {
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        Unauthorized,
        SelfTransfer,
        InsufficientAllowance,
        ContractPaused,
        Blacklisted,
        BatchLengthMismatch,
        Overflow,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        to: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        from: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Paused {
        is_paused: bool,
    }

    #[ink(event)]
    pub struct BlacklistUpdated {
        #[ink(topic)]
        account: AccountId,
        is_blacklisted: bool,
    }

    #[ink(storage)]
    pub struct Token {
        balances: Mapping<AccountId, u128>,
        owner: AccountId,
        total_supply: u128,
        allowances: Mapping<(AccountId, AccountId), u128>,
        paused: bool,
        blacklist: Mapping<AccountId, bool>,
    }

    impl Default for Token {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Token {
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
        pub fn mint(&mut self, to: AccountId, amount: u128) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            let current_balance = self.balances.get(to).unwrap_or(0);
            let new_balance = current_balance.checked_add(amount).ok_or(Error::Overflow)?;
            self.balances.insert(to, &new_balance);

            self.total_supply = self.total_supply.checked_add(amount).ok_or(Error::Overflow)?;
            self.env().emit_event(Mint { to, amount });

            Ok(())
        }

        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            self.balances.get(account).unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let from = self.env().caller();

            if self.paused {
                return Err(Error::ContractPaused);
            }

            if self.blacklist.get(from).unwrap_or(false) || self.blacklist.get(to).unwrap_or(false) {
                return Err(Error::Blacklisted);
            }

            if from == to {
                return Err(Error::SelfTransfer);
            }

            let from_balance = self.balances.get(from).unwrap_or(0);
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let to_balance = self.balances.get(to).unwrap_or(0);
            let new_from_balance = from_balance.checked_sub(amount).ok_or(Error::Overflow)?;
            let new_to_balance = to_balance.checked_add(amount).ok_or(Error::Overflow)?;
            self.balances.insert(from, &new_from_balance);
            self.balances.insert(to, &new_to_balance);

            self.env().emit_event(Transfer { from, to, amount });

            Ok(())
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        #[ink(message)]
        pub fn burn(&mut self, amount: u128) -> Result<()> {
            let from = self.env().caller();
            let from_balance = self.balances.get(from).unwrap_or(0);

            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let new_balance = from_balance.checked_sub(amount).ok_or(Error::Overflow)?;
            self.balances.insert(from, &new_balance);
            self.total_supply = self.total_supply.checked_sub(amount).ok_or(Error::Overflow)?;

            self.env().emit_event(Burn { from, amount });

            Ok(())
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, amount: u128) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), &amount);
            self.env().emit_event(Approval { owner, spender, amount });

            Ok(())
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, amount: u128) -> Result<()> {
            let spender = self.env().caller();

            if self.paused {
                return Err(Error::ContractPaused);
            }

            if self.blacklist.get(from).unwrap_or(false) || self.blacklist.get(to).unwrap_or(false) {
                return Err(Error::Blacklisted);
            }

            if from == to {
                return Err(Error::SelfTransfer);
            }

            let allowance = self.allowances.get((from, spender)).unwrap_or(0);
            if allowance < amount {
                return Err(Error::InsufficientAllowance);
            }

            let from_balance = self.balances.get(from).unwrap_or(0);
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let to_balance = self.balances.get(to).unwrap_or(0);
            let new_from_balance = from_balance.checked_sub(amount).ok_or(Error::Overflow)?;
            let new_to_balance = to_balance.checked_add(amount).ok_or(Error::Overflow)?;
            self.balances.insert(from, &new_from_balance);
            self.balances.insert(to, &new_to_balance);

            let new_allowance = allowance.checked_sub(amount).ok_or(Error::Overflow)?;
            self.allowances.insert((from, spender), &new_allowance);

            self.env().emit_event(Transfer { from, to, amount });

            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        #[ink(message)]
        pub fn pause(&mut self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            self.paused = true;
            self.env().emit_event(Paused { is_paused: true });

            Ok(())
        }

        #[ink(message)]
        pub fn unpause(&mut self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            self.paused = false;
            self.env().emit_event(Paused { is_paused: false });

            Ok(())
        }

        #[ink(message)]
        pub fn is_paused(&self) -> bool {
            self.paused
        }

        #[ink(message)]
        pub fn blacklist_account(&mut self, account: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            self.blacklist.insert(account, &true);
            self.env().emit_event(BlacklistUpdated { account, is_blacklisted: true });

            Ok(())
        }

        #[ink(message)]
        pub fn remove_from_blacklist(&mut self, account: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            self.blacklist.insert(account, &false);
            self.env().emit_event(BlacklistUpdated { account, is_blacklisted: false });

            Ok(())
        }

        #[ink(message)]
        pub fn is_blacklisted(&self, account: AccountId) -> bool {
            self.blacklist.get(account).unwrap_or(false)
        }

        #[ink(message)]
        pub fn batch_transfer(&mut self, recipients: ink::prelude::vec::Vec<AccountId>, amounts: ink::prelude::vec::Vec<u128>) -> Result<()> {
            if recipients.len() != amounts.len() {
                return Err(Error::BatchLengthMismatch);
            }

            let from = self.env().caller();

            if self.paused {
                return Err(Error::ContractPaused);
            }

            if self.blacklist.get(from).unwrap_or(false) {
                return Err(Error::Blacklisted);
            }

            let mut total_amount: u128 = 0;
            for amount in &amounts {
                total_amount = total_amount.checked_add(*amount).ok_or(Error::Overflow)?;
            }

            let from_balance = self.balances.get(from).unwrap_or(0);
            if from_balance < total_amount {
                return Err(Error::InsufficientBalance);
            }

            for (i, recipient) in recipients.iter().enumerate() {
                let amount = amounts[i];

                if self.blacklist.get(*recipient).unwrap_or(false) || from == *recipient {
                    continue;
                }

                let to_balance = self.balances.get(*recipient).unwrap_or(0);
                let new_to_balance = to_balance.checked_add(amount).ok_or(Error::Overflow)?;
                self.balances.insert(*recipient, &new_to_balance);

                self.env().emit_event(Transfer { from, to: *recipient, amount });
            }

            let new_from_balance = from_balance.checked_sub(total_amount).ok_or(Error::Overflow)?;
            self.balances.insert(from, &new_from_balance);

            Ok(())
        }
    }
}
