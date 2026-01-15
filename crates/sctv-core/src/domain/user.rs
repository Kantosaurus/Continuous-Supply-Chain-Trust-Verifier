//! User domain model for authentication and authorization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TenantId;

/// Unique identifier for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Creates a new random user ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Role of a user within a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Regular team member with read access.
    Member,
    /// Can manage projects and policies.
    Developer,
    /// Can manage users and tenant settings.
    Admin,
    /// Full access including billing and tenant deletion.
    Owner,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Member
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Member => write!(f, "member"),
            Self::Developer => write!(f, "developer"),
            Self::Admin => write!(f, "admin"),
            Self::Owner => write!(f, "owner"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "member" => Ok(Self::Member),
            "developer" => Ok(Self::Developer),
            "admin" => Ok(Self::Admin),
            "owner" => Ok(Self::Owner),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

/// A user account within a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub email: String,
    pub name: Option<String>,
    pub role: UserRole,
    pub api_key_hash: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a new user with the given email.
    #[must_use]
    pub fn new(tenant_id: TenantId, email: String) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            tenant_id,
            email,
            name: None,
            role: UserRole::default(),
            api_key_hash: None,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new user with a specific role.
    #[must_use]
    pub fn with_role(tenant_id: TenantId, email: String, role: UserRole) -> Self {
        let mut user = Self::new(tenant_id, email);
        user.role = role;
        user
    }

    /// Checks if the user has admin privileges.
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Owner)
    }

    /// Checks if the user can manage projects.
    #[must_use]
    pub const fn can_manage_projects(&self) -> bool {
        matches!(
            self.role,
            UserRole::Developer | UserRole::Admin | UserRole::Owner
        )
    }

    /// Checks if the user can manage other users.
    #[must_use]
    pub const fn can_manage_users(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Owner)
    }
}
