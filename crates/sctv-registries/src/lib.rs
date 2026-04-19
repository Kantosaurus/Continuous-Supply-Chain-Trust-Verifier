//! # SCTV Registries
//!
//! Package registry clients for the Supply Chain Trust Verifier.
//! Supports npm, PyPI, Maven, NuGet, RubyGems, Cargo, and Go modules.

mod cache;
mod retry;
mod traits;

pub mod cargo;
pub mod go_modules;
pub mod maven;
pub mod npm;
pub mod nuget;
pub mod pypi;
pub mod rubygems;

pub use cache::*;
pub use retry::{retry_http, RetryConfig};
pub use traits::*;
