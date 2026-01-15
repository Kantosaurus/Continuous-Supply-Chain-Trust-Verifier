//! Go module proxy client.
//!
//! This module provides a client for interacting with Go module proxies
//! (primarily proxy.golang.org) to fetch module metadata, versions, and downloads.
//!
//! # Module Proxy Protocol
//!
//! The Go module proxy protocol defines the following endpoints:
//!
//! - `GET /{module}/@v/list` - List available versions
//! - `GET /{module}/@v/{version}.info` - Get version info (JSON)
//! - `GET /{module}/@v/{version}.mod` - Get go.mod file
//! - `GET /{module}/@v/{version}.zip` - Download module source
//!
//! # Path Encoding
//!
//! Module paths with uppercase letters are encoded by prefixing each uppercase
//! letter with `!` and converting it to lowercase.
//! Example: `github.com/Azure/azure-sdk` becomes `github.com/!azure/azure-sdk`
//!
//! # Version Format
//!
//! Go modules use semantic versioning with a `v` prefix:
//! - `v1.2.3` - Standard semver
//! - `v1.2.3-rc1` - Prerelease
//! - `v1.2.3+incompatible` - For modules not using Go modules

mod client;
mod models;

pub use client::GoModulesClient;
pub use models::*;
