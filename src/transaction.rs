use serde::Deserialize;

// Type of transactions enum
// Using aliasis in case first leter of transaction type is lowercase
#[derive(Clone, Copy, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionEnum {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

// Holds all the information for a transaction
#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Transaction {
    // Transaction type
    #[serde(rename = "type")]
    pub tx_type: TransactionEnum,
    // Client ID
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    // Transaction ID
    pub tx_id: u32,
    #[serde(rename = "amount")]
    #[serde(default = "default_amount")]
    // Transaction amount
    pub tx_amount: f32,
}

impl Transaction {
    /// Returns a copy of internal transaction data, including
    /// the transaction type and amount.
    fn get_data(&self) -> (TransactionEnum, f32) {
        (self.tx_type, self.tx_amount)
    }
}

/// Used for dispute, resolve, chargeback transactions because they
/// don't include the amount field.
fn default_amount() -> f32 {
    f32::default()
}

#[cfg(test)]
mod tests {
    use super::{Transaction, TransactionEnum};

    #[test]
    fn retrieve_data_1() {
        let tx = Transaction {
            tx_type: TransactionEnum::Deposit,
            client_id: 01,
            tx_id: 02032,
            tx_amount: 123.34,
        };
        assert_eq!((TransactionEnum::Deposit, 123.34), tx.get_data());
    }
}
