//! # SCTV Core
//!
//! Core domain models and business logic for the Supply Chain Trust Verifier.
//! This crate defines all the fundamental types used throughout the system.

pub mod domain;
pub mod events;
pub mod traits;

pub use domain::*;
