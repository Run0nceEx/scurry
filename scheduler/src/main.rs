#[macro_use] extern crate diesel;

// behavioral
mod schedule;
mod miner;
mod error;

// persistence components
mod database;

// ephemeral persistence components maybe eventually?



fn main() {
    database::establish_connection("temp.db".to_string());
}