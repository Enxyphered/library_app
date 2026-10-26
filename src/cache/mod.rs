use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use moka::sync::Cache;

use crate::db::models::DashboardMetrics;

/// Monotonically increasing revision counters for domain entities.
///
/// Bumping a counter immediately invalidates all cache entries constructed
/// with the previous generation tag in O(1) time without lock contention.
#[derive(Debug, Default)]
pub struct EntityGenerations {
    pub books: AtomicU64,
    pub members: AtomicU64,
    pub loans: AtomicU64,
}

impl EntityGenerations {
    #[inline]
    pub fn bump_books(&self) -> u64 {
        self.books.fetch_add(1, Ordering::Release) + 1
    }

    #[inline]
    pub fn bump_members(&self) -> u64 {
        self.members.fetch_add(1, Ordering::Release) + 1
    }

    #[inline]
    #[allow(dead_code)]
    pub fn bump_loans(&self) -> u64 {
        self.loans.fetch_add(1, Ordering::Release) + 1
    }

    /// Atomically bumps both circulation-dependent entities (loans and books)
    /// because issuing or returning a loan changes books.available_copies.
    #[inline]
    pub fn bump_circulation(&self) {
        self.loans.fetch_add(1, Ordering::Release);
        self.books.fetch_add(1, Ordering::Release);
    }

    #[inline]
    pub fn dashboard_key(&self) -> String {
        format!(
            "dashboard:b{}:m{}:l{}",
            self.books.load(Ordering::Acquire),
            self.members.load(Ordering::Acquire),
            self.loans.load(Ordering::Acquire)
        )
    }

    #[inline]
    pub fn books_key(&self, subkey: &str) -> String {
        format!("books:v{}:{}", self.books.load(Ordering::Acquire), subkey)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn members_key(&self, subkey: &str) -> String {
        format!("members:v{}:{}", self.members.load(Ordering::Acquire), subkey)
    }
}

/// Unified bounded in-memory cache registered in Topcoat's `app_context(cx)`.
#[derive(Clone)]
pub struct AppCache {
    pub versions: Arc<EntityGenerations>,
    pub metrics: Cache<String, Arc<DashboardMetrics>>,
    pub categories: Cache<String, Arc<Vec<String>>>,
}

impl AppCache {
    pub fn new(_max_entries: u64, ttl_secs: u64) -> Self {
        Self {
            versions: Arc::new(EntityGenerations::default()),
            metrics: Cache::builder()
                .max_capacity(100)
                .time_to_idle(Duration::from_secs(ttl_secs))
                .build(),
            categories: Cache::builder()
                .max_capacity(100)
                .time_to_idle(Duration::from_secs(ttl_secs))
                .build(),
        }
    }
}
