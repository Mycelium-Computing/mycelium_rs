#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std_runtime"), no_std)]

pub mod futures {
    pub use futures::*;
}

pub mod async_lock {
    pub use ::async_lock::*;
}

pub mod core;
pub mod runtime_context;
pub mod runtimes;
pub mod utils;
pub use mycelium_computing_macros::*;

extern crate self as mycelium_computing;
