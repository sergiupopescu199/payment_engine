mod client;
mod process;
mod transaction;
use crate::transaction::Transaction;

use std::fs::File;

use anyhow::{bail, Context, Result};
use csv::{ByteRecord, Reader, ReaderBuilder, Trim, Writer};

lazy_static::lazy_static! {
    // Deposits and Withdrawals have 4 inputs
    pub static ref FOUR_INPUTS: ByteRecord = ByteRecord::from(
        vec!["type", "client", "tx", "amount"]
    );

    // Disputes, Resolves, and Chargebacks have 3 inputs
    pub static ref THREE_INPUTS: ByteRecord = ByteRecord::from(
        vec!["type", "client", "tx"]
    );
}

/// Opens file name read from command line.
/// Returns CSV parser
pub fn initialize() -> Result<Reader<File>> {
    match File::open(
        std::env::args()
            .nth(1)
            .context("Unable to get arguments, file.csv expected as argument")?,
    ) {
        Ok(file) => {
            return Ok(ReaderBuilder::new()
                .delimiter(b',')
                .flexible(true)
                .trim(Trim::All)
                .from_reader(file))
        }
        Err(e) => bail!(e),
    };
}

/// Processes transactions from file and print to stdout the account's balances as result
pub async fn process_txs(mut reader: Reader<File>) -> Result<()> {
    // crate a new instance of a ProcessTransaction task, it will handle all the logic by calculating the balances based on transaction type
    // it also will display the as tdout the result of its calculations
    let mut process_tx = process::ProcessTransactions::new();
    let mut record = ByteRecord::new();

    while reader.read_byte_record(&mut record)? {
        // for every record we must ensure it has the right amount of inputs on the line
        let tx: Transaction = record.deserialize(match record.len() {
            3 => Some(&THREE_INPUTS),
            4 => Some(&FOUR_INPUTS),
            _ => {
                bail!("Error reading data, invalid length of {}.", record.len())
            }
        })?;
        // send every record to ProcessTransaction task in the same order as it is read from the file
        let _ = process_tx.tx_tx.send(tx);
    }
    // after the file has been read completly drop the channel, in this way
    // the ProcessTransaction task will proceed evaluating transactions
    drop(process_tx.tx_tx);

    // create a CSV writer
    let mut writer = Writer::from_writer(std::io::stdout());
    // Write the header values to the record to printout in the output
    _ = writer.write_record(&["client", "available", "held", "total", "locked"]);

    // write every record received from ProcessTransaction task to stdout
    while let Some(record) = process_tx.rx_result.recv().await {
        if let Err(err) = writer.write_byte_record(&record) {
            {
                log::error!("Error in writing records! \n {err}")
            }
        }
    }
    Ok(())
}
