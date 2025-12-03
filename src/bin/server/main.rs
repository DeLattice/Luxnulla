use eyre::Error;
use mimalloc::MiMalloc;

use crate::{services::db::DbConnection, utils::config::AppPaths};

mod common;
mod http;
mod services;
mod utils;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> eyre::Result<(), Error> {
    AppPaths::init();

    let db = DbConnection::new()?;
    db.init_schema()?;

    http::server::init().await.unwrap();

    Ok(())
}
