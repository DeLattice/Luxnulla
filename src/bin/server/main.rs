use dirs::config_dir;
use eyre::{Error, OptionExt};
use luxnulla::{CONFIG_DIR, XRAY_CONFIG_FILE};
use mimalloc::MiMalloc;

use crate::services::db::DbConnection;

mod common;
mod handlers;
mod http;
mod services;
mod utils;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> eyre::Result<(), Error> {
    let config_dir_path = config_dir()
        .ok_or_eyre("cannot get a dir")
        .unwrap()
        .join(CONFIG_DIR);

    if !config_dir_path.exists() {
        std::fs::create_dir(&config_dir_path)?;
    }

    if !config_dir_path.join(XRAY_CONFIG_FILE).exists() {
        std::fs::File::create(&config_dir_path.join(XRAY_CONFIG_FILE)).unwrap();
    }

    let db = DbConnection::new()?;
    db.init_schema()?;

    http::server::init().await.unwrap();

    Ok(())
}
