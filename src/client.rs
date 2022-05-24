use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use tinyset::SetU32;

use crate::transaction::TransactionEnum;

#[derive(Debug, PartialEq, Clone)]
/// Represents client's account data
pub(crate) struct Client {
    /// Available balance
    balance_available: f32,
    /// Held balance
    balance_held: f32,
    /// Total balance
    balance_total: f32,
    /// Client's transactions
    transactions: HashMap<u32, (TransactionEnum, f32)>,
    /// List of disputed transactions
    disputed_tx: SetU32,
    /// Previous transaction ID
    previous_tx_id: u32,
    /// Flag indicating if account is frozen (chargeback)
    frozen: bool,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            balance_available: 0.0,
            balance_held: 0.0,
            balance_total: 0.0,
            transactions: HashMap::new(),
            disputed_tx: SetU32::new(),
            previous_tx_id: 0,
            frozen: false,
        }
    }
}

impl Client {
    /// Returns a new client
    pub(crate) fn new(tx_id: u32, tx_type: TransactionEnum, tx_amount: f32) -> Self {
        let balance = {
            match tx_type {
                TransactionEnum::Deposit => tx_amount,
                _ => 0.0,
            }
        };

        let mut client = Client {
            balance_available: balance,
            balance_total: balance,
            previous_tx_id: tx_id,
            ..Default::default()
        };

        client.chain_tx(tx_id, tx_type, tx_amount);
        return client;
    }

    /// Checks if the account is currently frozen.
    ///
    /// Returns `true` if it's frozen
    pub(crate) fn account_frozen(&self, tx_id: u32) -> Result<bool> {
        if self.frozen {
            Err(anyhow!(
                "Account is currently frozen, transaction ID: {} was not processed!",
                tx_id
            ))
        } else {
            return Ok(self.frozen);
        }
    }

    /// Store current transaction and chain it to the previous one
    pub(crate) fn chain_tx(&mut self, tx_id: u32, tx_type: TransactionEnum, tx_amount: f32) {
        self.previous_tx_id = tx_id;
        self.transactions.insert(tx_id, (tx_type, tx_amount));
    }

    /// Checks if there is sufficient funds available to process transaction
    pub(crate) fn sufficient_funds(&self, tx_amount: f32) -> Result<()> {
        if self.balance_available >= tx_amount {
            return Ok(());
        }
        bail!(
            "Not enough available balance to process withdrawal!
        \n Balance: {}
        \n Withdrawal Amount: {tx_amount}",
            self.balance_available
        );
    }

    /// Checks the disputed status of a past transaction, and compare
    /// it to the value passed into the call
    pub(crate) fn disputed_status(&self, tx_id: u32, status: bool) -> Result<()> {
        if self.disputed_tx.contains(tx_id) == status {
            return Ok(());
        }
        bail!("Transaction ID: {tx_id} is already labeled as disputed!");
    }

    /// Search the logs for the given transaction ID and if found return value of it
    ///
    /// Only transaction of type `Deposit` and `Withdrawal` have values others don't
    pub fn get_tx_val(&self, tx_id: u32) -> Result<f32> {
        match self.transactions.get(&tx_id) {
            Some((_, tx_amount)) => return Ok(tx_amount.to_owned()),
            None => bail!("Failed to get value! Transaction ID: {tx_id} does not exist!"),
        }
    }

    /// Processes the current transaction based on it's type
    pub(crate) fn process_tx(
        &mut self,
        tx_id: u32,
        tx_type: TransactionEnum,
        tx_amount: f32,
    ) -> Result<()> {
        self.account_frozen(tx_id)?;

        match tx_type {
            // increase balance on a client a account
            TransactionEnum::Deposit => {
                self.balance_available += tx_amount;
                self.balance_total = self.balance_available + self.balance_held;
                self.chain_tx(tx_id, tx_type, tx_amount);
            }
            // If client does not have suffecient funds available, the withdraw will fail
            // and the account's state will remain unchanged.
            TransactionEnum::Withdrawal => {
                self.sufficient_funds(tx_amount)?;
                self.balance_available -= tx_amount;
                self.balance_total = self.balance_available + self.balance_held;
                self.chain_tx(tx_id, tx_type, tx_amount);
            }
            // If the transaction ID is valid, held funds will increase and
            // available balance will decrease by the funds asscociated to the
            // provided transaction ID.
            TransactionEnum::Dispute => {
                self.disputed_status(tx_id, false)?;
                let disputed_val = self.get_tx_val(tx_id)?;
                self.sufficient_funds(disputed_val)?;
                self.balance_available -= disputed_val;
                self.balance_held += disputed_val;
                self.disputed_tx.insert(tx_id);
            }
            // If the transaction ID is valid and it is under dispute, held
            // funds will decrease and available balance will increase by the
            // funds asscociated to the provided transaction ID.
            TransactionEnum::Resolve => {
                self.disputed_status(tx_id, true)?;
                let disputed_val = self.get_tx_val(tx_id)?;
                if disputed_val <= self.balance_held {
                    self.balance_available += disputed_val;
                    self.balance_held -= disputed_val;
                    self.disputed_tx.remove(tx_id);
                }
            }
            // If the transaction ID is valid and it is under dispute, funds
            // that were held will be withdrawn.
            // Held funds and total funds will decrease by the funds previously
            // disputed.
            TransactionEnum::Chargeback => {
                self.disputed_status(tx_id, true)?;
                let disputed_val = self.get_tx_val(tx_id)?;
                if disputed_val <= self.balance_held {
                    self.frozen = true;
                    self.balance_held -= disputed_val;
                    self.balance_total -= disputed_val;
                    self.disputed_tx.remove(tx_id);
                }
            }
        }
        Ok(())
    }

    /// Retrieves client's account infomation
    pub(crate) fn get_info(&self, client_id: &u16) -> Vec<String> {
        vec![
            client_id.to_string(),
            format!("{:.4}", self.balance_available),
            format!("{:.4}", self.balance_held),
            format!("{:.4}", self.balance_total),
            self.frozen.to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn client_creation() {
        let client1 = Client::new(123456, TransactionEnum::Deposit, 5000.1234);

        let mut tx_log: HashMap<u32, (TransactionEnum, f32)> = HashMap::new();
        tx_log.insert(5546465, (TransactionEnum::Deposit, 5000.1234));

        let client2 = Client {
            balance_available: 5000.1234,
            balance_held: 0.0,
            balance_total: 5000.1234,
            transactions: tx_log,
            disputed_tx: SetU32::new(),
            previous_tx_id: 123456,
            frozen: false,
        };
        assert_eq!(client1, client2);
    }
}
