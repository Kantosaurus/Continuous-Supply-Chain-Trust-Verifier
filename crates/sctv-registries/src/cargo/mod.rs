//! Crates.io registry client.
//!
//! This module provides a client for interacting with the crates.io API
//! to fetch Rust crate metadata, versions, and downloads.
//!
//! # API Endpoints
//!
//! - Package metadata: `GET /api/v1/crates/{name}`
//! - Version metadata: `GET /api/v1/crates/{name}/{version}`
//! - Dependencies: `GET /api/v1/crates/{name}/{version}/dependencies`
//! - Owners: `GET /api/v1/crates/{name}/owners`
//! - Download: `https://static.crates.io/crates/{name}/{name}-{version}.crate`

mod client;
mod models;

pub use client::CargoClient;
pub use models::*;
