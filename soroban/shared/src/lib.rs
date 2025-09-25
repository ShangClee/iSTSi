#![no_std]

//! Shared utilities for Bitcoin Custody Soroban contracts
//! 
//! This crate provides common functionality used across multiple contracts
//! in the Bitcoin custody system, including types, error handling, and
//! utility functions.

pub mod types;
pub mod errors;
pub mod utils;
pub mod events;

// Re-export commonly used items
pub use types::*;
pub use errors::*;
pub use utils::*;
pub use events::*;