#![allow(non_snake_case)]
#![feature(async_closure)]

use u2client::client::U2client;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod torrentLib;
pub mod u2client;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
