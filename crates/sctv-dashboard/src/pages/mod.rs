//! Page components for the SCTV Dashboard.
//!
//! Each page implements a distinct view with bold, asymmetric layouts
//! that maintain visual consistency while serving unique purposes.

mod alerts;
mod not_found;
mod policies;
mod projects;
mod settings;

pub use alerts::AlertsPage;
pub use not_found::NotFoundPage;
pub use policies::PoliciesPage;
pub use projects::ProjectsPage;
pub use settings::SettingsPage;
