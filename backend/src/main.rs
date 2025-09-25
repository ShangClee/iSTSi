use bitcoin_custody_backend::app::App;
use loco_rs::cli;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    Ok(cli::main::<App, Migrator>().await?)
}