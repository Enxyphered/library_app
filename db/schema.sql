-- ============================================================================
-- Topcoat + HTMX Library Application: Production SQLite Database Schema
-- Architecture: High-Concurrency SQLite in WAL Mode with Strict Relational Integrity
-- ============================================================================

-- Connection Pragmas (Must be executed by every connection initialization)
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA cache_size = -64000;        -- 64MB RAM page cache
PRAGMA mmap_size = 268435456;      -- 256MB memory-mapped I/O
PRAGMA temp_store = MEMORY;        -- In-memory temporary tables & sorting
PRAGMA wal_autocheckpoint = 1000;  -- Checkpoint every 1000 pages (~4MB)

-- ----------------------------------------------------------------------------
-- 1. Schema Migration History Table
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS _migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version INTEGER NOT NULL UNIQUE,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ----------------------------------------------------------------------------
-- 2. Users Table (Librarians & System Administrators)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL COLLATE NOCASE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('admin', 'librarian')) DEFAULT 'librarian',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CONSTRAINT uq_users_username UNIQUE (username)
);

-- ----------------------------------------------------------------------------
-- 3. Books Table (Catalog & Inventory)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    isbn TEXT NOT NULL,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'General',
    total_copies INTEGER NOT NULL CHECK(total_copies >= 0),
    available_copies INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CONSTRAINT uq_books_isbn UNIQUE (isbn),
    CONSTRAINT chk_books_copies CHECK(
        available_copies >= 0 AND 
        available_copies <= total_copies
    )
);

-- ----------------------------------------------------------------------------
-- 4. Members Table (Patrons & Borrowers)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    membership_number TEXT NOT NULL COLLATE NOCASE,
    name TEXT NOT NULL,
    email TEXT NOT NULL COLLATE NOCASE,
    phone TEXT,
    status TEXT NOT NULL CHECK(status IN ('active', 'suspended')) DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CONSTRAINT uq_members_membership_number UNIQUE (membership_number),
    CONSTRAINT uq_members_email UNIQUE (email)
);

-- ----------------------------------------------------------------------------
-- 5. Loans Table (Circulation & Borrowing History)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS loans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE RESTRICT ON UPDATE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE RESTRICT ON UPDATE CASCADE,
    loan_date TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d', 'now')),
    due_date TEXT NOT NULL,
    return_date TEXT,
    status TEXT NOT NULL CHECK(status IN ('active', 'returned', 'overdue')) DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CONSTRAINT chk_loans_dates CHECK(return_date IS NULL OR return_date >= loan_date)
);

-- ----------------------------------------------------------------------------
-- 6. Audit & Activity Log Table
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ============================================================================
-- HIGH-PERFORMANCE INDEXES
-- Designed for sub-millisecond query plans, zero full-table scans, and index-only lookups
-- ============================================================================

-- Books Indexes
CREATE UNIQUE INDEX IF NOT EXISTS idx_books_isbn ON books(isbn);
CREATE INDEX IF NOT EXISTS idx_books_title ON books(title COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_books_author ON books(author COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_books_category_title ON books(category, title COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_books_available ON books(available_copies) WHERE available_copies > 0;

-- Members Indexes
CREATE UNIQUE INDEX IF NOT EXISTS idx_members_membership_number ON members(membership_number COLLATE NOCASE);
CREATE UNIQUE INDEX IF NOT EXISTS idx_members_email ON members(email COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_members_name ON members(name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_members_status ON members(status);

-- Loans Indexes (Composite & Partial)
-- 1. Partial Index for Overdue Sweeps and Active Loan Dashboards:
-- Excludes 'returned' rows (which constitute ~90% of loan records over time).
CREATE INDEX IF NOT EXISTS idx_loans_active_due_date 
ON loans(status, due_date) 
WHERE status != 'returned';

-- 2. Foreign Key Indexes for Fast Member History & Book Status:
CREATE INDEX IF NOT EXISTS idx_loans_member_status ON loans(member_id, status);
CREATE INDEX IF NOT EXISTS idx_loans_book_status ON loans(book_id, status);

-- Audit Log Timeline Index
CREATE INDEX IF NOT EXISTS idx_audit_logs_timeline ON audit_logs(created_at DESC);
