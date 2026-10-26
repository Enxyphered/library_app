//! Strongly-typed domain models for the Library Management System.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Librarian,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::Librarian => "librarian",
        }
    }
}

impl TryFrom<&str> for UserRole {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "librarian" => Ok(UserRole::Librarian),
            other => Err(format!("Unknown user role: {other}")),
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub id: i64,
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub total_copies: i32,
    pub available_copies: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl Book {
    #[inline]
    #[allow(dead_code)]
    pub fn is_available(&self) -> bool {
        self.available_copies > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    pub id: i64,
    pub membership_number: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub status: MemberStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberStatus {
    Active,
    Suspended,
}

impl MemberStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberStatus::Active => "active",
            MemberStatus::Suspended => "suspended",
        }
    }
}

impl TryFrom<&str> for MemberStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "active" => Ok(MemberStatus::Active),
            "suspended" => Ok(MemberStatus::Suspended),
            other => Err(format!("Unknown member status: {other}")),
        }
    }
}

impl std::fmt::Display for MemberStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loan {
    pub id: i64,
    pub book_id: i64,
    pub member_id: i64,
    pub loan_date: String,
    pub due_date: String,
    pub return_date: Option<String>,
    pub status: LoanStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoanStatus {
    Active,
    Returned,
    Overdue,
}

impl LoanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoanStatus::Active => "active",
            LoanStatus::Returned => "returned",
            LoanStatus::Overdue => "overdue",
        }
    }
}

impl TryFrom<&str> for LoanStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "active" => Ok(LoanStatus::Active),
            "returned" => Ok(LoanStatus::Returned),
            "overdue" => Ok(LoanStatus::Overdue),
            other => Err(format!("Unknown loan status: {other}")),
        }
    }
}

impl std::fmt::Display for LoanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Rich loan detail view with joined book and member data for circulation tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoanWithDetails {
    pub loan_id: i64,
    pub book_id: i64,
    pub book_title: String,
    pub book_isbn: String,
    pub member_id: i64,
    pub member_number: String,
    pub member_name: String,
    pub member_email: String,
    pub loan_date: String,
    pub due_date: String,
    pub return_date: Option<String>,
    pub status: LoanStatus,
    pub days_overdue: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DashboardMetrics {
    pub total_titles: i64,
    pub total_physical_copies: i64,
    pub available_physical_copies: i64,
    pub active_loans_count: i64,
    pub overdue_loans_count: i64,
    pub total_active_members: i64,
}
