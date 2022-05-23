use anyhow::Result;
use payment_engine::{initialize, process_txs};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    process_txs(initialize()?).await?;
    Ok(())
}
