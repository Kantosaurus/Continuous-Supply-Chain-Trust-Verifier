//! NuGet registry client.
//!
//! This module provides a client for interacting with the NuGet API v3
//! to fetch .NET package metadata, versions, and downloads.
//!
//! # API v3 Discovery
//!
//! NuGet v3 uses a service index for discovering API endpoints:
//! - Service index: `https://api.nuget.org/v3/index.json`
//!
//! From the service index, we discover:
//! - Registration: Package metadata and versions
//! - PackageBaseAddress: Package downloads (.nupkg files)
//! - SearchQueryService: Package search

mod client;
mod models;

pub use client::NuGetClient;
pub use models::*;
