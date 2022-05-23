use csv::ByteRecord;
use std::collections::HashMap;
use tokio::sync::{mpsc, mpsc::error::TryRecvError};

use crate::{client::Client, transaction::Transaction};
/// This task processes transactions, for every transaction received
/// it sorts them by client id and performs actions based on transaction type for every client id.
struct ProcessTransactionsTask {
    /// receive a transaction from high level
    rx_tx: mpsc::UnboundedReceiver<Transaction>,
    /// send client info
    tx_result: mpsc::UnboundedSender<ByteRecord>,
    /// store client ids and its data based on transactrions it receives
    clients: HashMap<u16, Client>,
}

impl ProcessTransactionsTask {
    /// run the task
    pub async fn run(&mut self) {
        // loop while channel is not disconected
        loop {
            match self.rx_tx.try_recv() {
                Ok(tx) => {
                    self.clients
                        // create a new entry using client's id from transaction
                        .entry(tx.client_id)
                        // if the given entry has a client instance already set as value modify
                        // the data based on the new transactions it receives
                        .and_modify(|client| {
                            if let Err(err) = client.process_tx(tx.tx_id, tx.tx_type, tx.tx_amount)
                            {
                                log::error!("Error processing transaction! {tx:?}\n{err}")
                            }
                        })
                        // if there's value associated to the current client id entry create a new client
                        .or_insert_with(|| Client::new(tx.tx_id, tx.tx_type, tx.tx_amount));
                }
                Err(TryRecvError::Disconnected) => {
                    // after channel was dropped we can proceed to send out to high level the
                    // account balances
                    return self.send_acccount_balances();
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }

    /// send account balances to high level
    pub fn send_acccount_balances(&self) {
        // for every client id get it's info and send it to high level
        self.clients.iter().for_each(|(client_id, client)| {
            let _ = self
                .tx_result
                .send(ByteRecord::from(client.get_info(client_id)));
        });
    }
}

/// Process transactions and get client balance information
pub struct ProcessTransactions {
    /// Send a new transaction to be processed
    pub tx_tx: mpsc::UnboundedSender<Transaction>,
    /// Receive client's balance information based on it's transaction flow
    pub rx_result: mpsc::UnboundedReceiver<ByteRecord>,
}

impl ProcessTransactions {
    pub fn new() -> Self {
        // create channels needed for comunication
        let (tx_tx, rx_tx) = mpsc::unbounded_channel();
        let (tx_result, rx_result) = mpsc::unbounded_channel();

        // spawn a new task in background, it lives as long as ProcessTransaction
        tokio::spawn(async move {
            ProcessTransactionsTask {
                rx_tx,
                tx_result,
                clients: HashMap::new(),
            }
            .run()
            .await
        });

        Self { tx_tx, rx_result }
    }
}
