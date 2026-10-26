//! SQLite High-Performance Database Architecture & Connection Management
//!
//! Features:
//! - WAL (Write-Ahead Logging) mode for concurrent readers + single writer.
//! - synchronous = NORMAL for optimal commit throughput with zero crash corruption.
//! - foreign_keys = ON enforced per connection.
//! - busy_timeout = 5000ms eliminating concurrency locking contention.
//! - 64MB memory page cache & 256MB memory-mapped I/O (mmap).
//! - r2d2 connection pool integrated with Tokio's `spawn_blocking` thread pool.
//! - Strict BEGIN IMMEDIATE transaction wrappers eliminating lock upgrade deadlocks.

pub mod circulation;
pub mod migrations;
pub mod models;
pub mod queries;

use std::path::{Path, PathBuf};
use std::time::Duration;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, OpenFlags, TransactionBehavior};
use thiserror::Error;

/// Database error taxonomy distinguishing concurrency and relational failures
#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Connection pool acquisition error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Tokio task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("Record not found: {entity} with identifier '{id}'")]
    NotFound { entity: &'static str, id: String },

    #[error("Book stock exhausted: Book ID {book_id} has 0 available copies")]
    InventoryExhausted { book_id: i64 },

    #[error("Member suspended: Member ID {member_id} is not permitted to borrow books")]
    MemberSuspended { member_id: i64 },

    #[error("Loan already returned: Loan ID {loan_id} is not in active or overdue status")]
    LoanAlreadyReturned { loan_id: i64 },

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Internal migration error: {0}")]
    Migration(String),
}

/// SQLite Connection Customizer enforcing high-performance and safety PRAGMAs
#[derive(Debug, Clone)]
pub struct SqlitePragmaCustomizer {
    pub busy_timeout: Duration,
    pub cache_size_kib: i32,
    pub mmap_size_bytes: i64,
}

impl Default for SqlitePragmaCustomizer {
    fn default() -> Self {
        Self {
            busy_timeout: Duration::from_millis(5000),
            cache_size_kib: 64000,           // 64 MB cache
            mmap_size_bytes: 268_435_456,    // 256 MB mmap
        }
    }
}

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for SqlitePragmaCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        // 1. Write-Ahead Logging (readers never block writers, writers never block readers)
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // 2. Normal synchronous: commits sync to WAL ringbuffer, fsync at checkpoint
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        // 3. Foreign key enforcement (crucial: must be set on EVERY connection)
        conn.pragma_update(None, "foreign_keys", "ON")?;

        // 4. Busy timeout: sleep & retry automatically up to 5,000ms instead of failing
        let timeout_ms = self.busy_timeout.as_millis() as i64;
        conn.pragma_update(None, "busy_timeout", timeout_ms)?;

        // 5. Memory Page Cache: negative value specifies KiB
        let cache_val = -self.cache_size_kib;
        conn.pragma_update(None, "cache_size", cache_val)?;

        // 6. Memory-Mapped I/O: 256 MB allows zero-syscall page reads from kernel buffer
        conn.pragma_update(None, "mmap_size", self.mmap_size_bytes)?;

        // 7. Store temp tables, indices, and sort operations in RAM
        conn.pragma_update(None, "temp_store", "MEMORY")?;

        // 8. Auto-checkpoint WAL every 1,000 pages (~4MB)
        conn.pragma_update(None, "wal_autocheckpoint", 1000)?;

        Ok(())
    }
}

/// Cloned handle to the SQLite connection pool, safe to share across Tokio tasks
#[derive(Clone)]
pub struct DbPool {
    pool: Pool<SqliteConnectionManager>,
    #[allow(dead_code)]
    path: PathBuf,
}

impl DbPool {
    /// Initialize a new connection pool for the given database file path.
    /// Creates the file and parent directory if they do not exist.
    pub fn new<P: AsRef<Path>>(path: P, max_connections: u32) -> Result<Self, DbError> {
        let path_buf = path.as_ref().to_path_buf();

        if let Some(parent) = path_buf.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DbError::Migration(format!("Failed to create database directory: {e}"))
                })?;
            }
        }

        let manager = SqliteConnectionManager::file(&path_buf)
            .with_flags(
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            );

        let customizer = Box::new(SqlitePragmaCustomizer::default());

        let pool = Pool::builder()
            .max_size(max_connections)
            .min_idle(Some(2.min(max_connections)))
            .connection_timeout(Duration::from_secs(5))
            .connection_customizer(customizer)
            .build(manager)?;

        let db = Self {
            pool,
            path: path_buf,
        };

        // Run idempotent migrations immediately upon pool bootstrap
        db.run_migrations_sync()?;

        Ok(db)
    }

    /// Construct an in-memory pool for isolated unit and integration testing
    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self, DbError> {
        let manager = SqliteConnectionManager::memory()
            .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE);

        let customizer = Box::new(SqlitePragmaCustomizer::default());

        let pool = Pool::builder()
            .max_size(1) // Single connection for in-memory DB preserves state
            .connection_customizer(customizer)
            .build(manager)?;

        let db = Self {
            pool,
            path: PathBuf::from(":memory:"),
        };

        db.run_migrations_sync()?;
        Ok(db)
    }

    /// Synchronously run pending migrations on pool startup
    fn run_migrations_sync(&self) -> Result<(), DbError> {
        let mut conn = self.pool.get()?;
        migrations::run_migrations(&mut conn)
    }

    /// Execute a read-only query offloaded to Tokio's blocking thread pool.
    /// Does not acquire a write lock; scales smoothly across pool connections.
    pub async fn read<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&Connection) -> Result<R, DbError> + Send + 'static,
        R: Send + 'static,
    {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            f(&conn)
        })
        .await?
    }

    /// Execute a write transaction offloaded to Tokio's blocking thread pool.
    ///
    /// CRITICAL ARCHITECTURE NOTE:
    /// Uses `TransactionBehavior::Immediate` (BEGIN IMMEDIATE).
    /// This immediately acquires the reserved SQLite write lock at the start
    /// of the transaction, rather than deferring until the first UPDATE/INSERT.
    /// This strictly prevents deadlocks and lock-upgrade race conditions
    /// under high concurrent request volumes.
    pub async fn write<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&mut rusqlite::Transaction) -> Result<R, DbError> + Send + 'static,
        R: Send + 'static,
    {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let mut tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let result = f(&mut tx)?;
            tx.commit()?;
            Ok(result)
        })
        .await?
    }

    /// Direct synchronous connection checkout for background workers or migrations
    #[allow(dead_code)]
    pub fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, DbError> {
        Ok(self.pool.get()?)
    }

    /// Get current database file path
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }
}
