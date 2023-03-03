#[cfg(not(target_arch = "wasm32"))]
mod tokio;

#[cfg(not(target_arch = "wasm32"))]
pub use self::tokio::*;
use std::fmt::Formatter;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::*;

#[derive(Debug)]
pub struct PoolError;

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "join error")
    }
}

impl std::error::Error for PoolError {}
