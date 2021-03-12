#![allow(non_snake_case)]
#![feature(async_closure)]

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod torrentLib;
pub mod u2client;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
