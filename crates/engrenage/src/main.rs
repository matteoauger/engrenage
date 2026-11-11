use clap::Parser;

use engrenage::{Args, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run(Args::parse()).await
}
