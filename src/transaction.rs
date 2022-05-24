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

/// Used for dispute, resolve, chargeback transactions because they
/// don't include the amount field.
fn default_amount() -> f32 {
    f32::default()
}

#[cfg(test)]
mod tests {

    use std::fs::File;

    use super::{Transaction, TransactionEnum};
    use anyhow::Result;
    use csv::{ByteRecord, Reader, ReaderBuilder, Trim};

    fn initialize() -> Result<Reader<File>> {
        match File::open("csv_files/tx_test.csv") {
            Ok(file) => {
                return Ok(ReaderBuilder::new()
                    .delimiter(b',')
                    .flexible(true)
                    .trim(Trim::All)
                    .from_reader(file))
            }
            Err(e) => panic!("{e}"),
        };
    }

    #[test]
    fn retrieve_data() {
        let mut reader = initialize().unwrap();
        let mut record = ByteRecord::new();
        let four_inputs = ByteRecord::from(vec!["type", "client", "tx", "amount"]);
        let three_inputs = ByteRecord::from(vec!["type", "client", "tx"]);

        let compare_tx = vec![
            Transaction {
                tx_type: TransactionEnum::Deposit,
                client_id: 1,
                tx_id: 1,
                tx_amount: 10.0,
            },
            Transaction {
                tx_type: TransactionEnum::Withdrawal,
                client_id: 1,
                tx_id: 4,
                tx_amount: 3.0,
            },
            Transaction {
                tx_type: TransactionEnum::Dispute,
                client_id: 1,
                tx_id: 3,
                tx_amount: 0.0,
            },
            Transaction {
                tx_type: TransactionEnum::Resolve,
                client_id: 1,
                tx_id: 3,
                tx_amount: 0.0,
            },
            Transaction {
                tx_type: TransactionEnum::Chargeback,
                client_id: 1,
                tx_id: 3,
                tx_amount: 0.0,
            },
        ];
        let mut store_tx = vec![];

        while reader.read_byte_record(&mut record).unwrap() {
            // for every record we must ensure it has the right amount of inputs on the line
            let tx: Transaction = record
                .deserialize(match record.len() {
                    4 => Some(&four_inputs),
                    3 => Some(&three_inputs),
                    _ => {
                        panic!("Error reading data, invalid length of {}.", record.len())
                    }
                })
                .unwrap();
            store_tx.push(tx);
        }

        store_tx
            .iter()
            .enumerate()
            .for_each(|(index, tx)| assert_eq!(tx, compare_tx.get(index).unwrap()))
    }
}
