//! Runtime-specific context implementations.
//!
//! Each runtime is enabled independently through its corresponding Cargo feature.

#[cfg(feature = "std_runtime")]
mod std_runtime;

#[cfg(feature = "std_runtime")]
pub use std_runtime::{StdMutex, StdRuntimeContext};
