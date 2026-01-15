//! RubyGems registry client.
//!
//! This module provides a client for interacting with the RubyGems API
//! to fetch Ruby gem metadata, versions, and downloads.
//!
//! # API Endpoints
//!
//! - Gem info: `GET /api/v1/gems/{name}.json`
//! - Versions: `GET /api/v1/versions/{name}.json`
//! - Owners: `GET /api/v1/gems/{name}/owners.json`
//! - Download: `https://rubygems.org/gems/{name}-{version}.gem`
//!
//! # Version Format
//!
//! Ruby gems use a version format that's mostly semver compatible, but may
//! have additional segments (e.g., "1.2.3.4"). This client handles conversion
//! to standard semver format.

mod client;
mod models;

pub use client::RubyGemsClient;
pub use models::*;
