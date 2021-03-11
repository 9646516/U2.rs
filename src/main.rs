#![allow(non_snake_case)]

use u2client::client::U2client;

pub mod torrentLib;
pub mod u2client;

#[tokio::main]
async fn main() -> u2client::Result<()> {
    Ok(())
}
