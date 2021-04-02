#![feature(test)]
#![feature(toowned_clone_into)]
pub mod error;
pub mod model;
pub mod pool;
pub mod util;

#[cfg(feature = "include-test")]
pub use pool::test as tests;