//! Idempotent database migrations and initial seed execution.

use crate::db::DbError;
use rusqlite::{params, Connection, TransactionBehavior};

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "001_core_schema",
        sql: r#"
        -- 1. Users
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL COLLATE NOCASE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('admin', 'librarian')) DEFAULT 'librarian',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            CONSTRAINT uq_users_username UNIQUE (username)
        );

        -- 2. Books
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

        -- 3. Members
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

        -- 4. Loans
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

        -- 5. Audit Log
        CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            action TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id INTEGER,
            details TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        -- High Performance Indexes
        CREATE UNIQUE INDEX IF NOT EXISTS idx_books_isbn ON books(isbn);
        CREATE INDEX IF NOT EXISTS idx_books_title ON books(title COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_books_author ON books(author COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_books_category_title ON books(category, title COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_books_available ON books(available_copies) WHERE available_copies > 0;

        CREATE UNIQUE INDEX IF NOT EXISTS idx_members_membership_number ON members(membership_number COLLATE NOCASE);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_members_email ON members(email COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_members_name ON members(name COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_members_status ON members(status);

        CREATE INDEX IF NOT EXISTS idx_loans_active_due_date ON loans(status, due_date) WHERE status != 'returned';
        CREATE INDEX IF NOT EXISTS idx_loans_member_status ON loans(member_id, status);
        CREATE INDEX IF NOT EXISTS idx_loans_book_status ON loans(book_id, status);

        CREATE INDEX IF NOT EXISTS idx_audit_logs_timeline ON audit_logs(created_at DESC);
        "#,
    },
    Migration {
        version: 2,
        name: "002_initial_seeds",
        sql: r#"
        -- 1. Default Admin and Librarian accounts
        -- admin / admin (Argon2id format)
        INSERT OR IGNORE INTO users (id, username, password_hash, role)
        VALUES 
          (1, 'admin', '$argon2id$v=19$m=19456,t=2,p=1$temUD3doq8QhVP1FKonI+g$YGpEM39YVkqjnGRvC1NFHVjXLiVl/bdx8/17JVS6O7A', 'admin'),
          (2, 'librarian', '$argon2id$v=19$m=19456,t=2,p=1$temUD3doq8QhVP1FKonI+g$YGpEM39YVkqjnGRvC1NFHVjXLiVl/bdx8/17JVS6O7A', 'librarian');

        -- 2. Core Book Catalog
        INSERT OR IGNORE INTO books (id, isbn, title, author, category, total_copies, available_copies)
        VALUES
          (1, '978-0132350884', 'Clean Code: A Handbook of Agile Software Craftsmanship', 'Robert C. Martin', 'Software Engineering', 5, 3),
          (2, '978-0596007126', 'Head First Design Patterns', 'Eric Freeman & Elisabeth Robson', 'Software Engineering', 4, 4),
          (3, '978-1449373320', 'Designing Data-Intensive Applications', 'Martin Kleppmann', 'Distributed Systems', 6, 4),
          (4, '978-0201616224', 'The Pragmatic Programmer: Your Journey to Mastery', 'David Thomas & Andrew Hunt', 'Software Engineering', 5, 2),
          (5, '978-1593278281', 'The Rust Programming Language', 'Steve Klabnik & Carol Nichols', 'Programming Languages', 7, 5),
          (6, '978-0262033848', 'Introduction to Algorithms (4th Edition)', 'Thomas H. Cormen et al.', 'Algorithms', 4, 3),
          (7, '978-0131103627', 'The C Programming Language (2nd Edition)', 'Brian W. Kernighan & Dennis M. Ritchie', 'Programming Languages', 3, 3),
          (8, '978-0321751041', 'The Art of Computer Programming, Vol 1', 'Donald E. Knuth', 'Algorithms', 2, 2),
          (9, '978-1491950357', 'Building Microservices: Designing Fine-Grained Systems', 'Sam Newman', 'Architecture', 5, 4),
          (10, '978-0321125217', 'Domain-Driven Design: Tackling Complexity', 'Eric Evans', 'Architecture', 4, 4),
          (11, '978-0385537858', 'The Phoenix Project', 'Gene Kim, Kevin Behr & George Spafford', 'DevOps & Management', 6, 5),
          (12, '978-1492040347', 'Programming Rust: Fast, Safe Systems Development', 'Jim Blandy et al.', 'Programming Languages', 5, 5);

        -- 3. Core Patrons / Members
        INSERT OR IGNORE INTO members (id, membership_number, name, email, phone, status)
        VALUES
          (1, 'LIB-2026-1001', 'Elena Rostova', 'elena.rostova@example.com', '+1-555-0192', 'active'),
          (2, 'LIB-2026-1002', 'Marcus Vance', 'marcus.vance@example.com', '+1-555-0144', 'active'),
          (3, 'LIB-2026-1003', 'Dr. Aris Thorne', 'aris.thorne@university.edu', '+1-555-0188', 'active'),
          (4, 'LIB-2026-1004', 'Clara O''Connor', 'clara.oconnor@example.com', '+1-555-0112', 'active'),
          (5, 'LIB-2026-1005', 'Julian Mercer', 'julian.mercer@example.com', '+1-555-0167', 'suspended');

        -- 4. Sample Loans (Active, Overdue, and Historic Returned)
        INSERT OR IGNORE INTO loans (id, book_id, member_id, loan_date, due_date, return_date, status)
        VALUES
          (1, 1, 1, date('now', '-7 days'),  date('now', '+7 days'),  NULL, 'active'),
          (2, 1, 2, date('now', '-20 days'), date('now', '-6 days'),  NULL, 'overdue'),
          (3, 3, 3, date('now', '-4 days'),  date('now', '+10 days'), NULL, 'active'),
          (4, 3, 4, date('now', '-18 days'), date('now', '-4 days'),  NULL, 'overdue'),
          (5, 4, 1, date('now', '-2 days'),  date('now', '+12 days'), NULL, 'active'),
          (6, 4, 2, date('now', '-3 days'),  date('now', '+11 days'), NULL, 'active'),
          (7, 4, 4, date('now', '-1 days'),  date('now', '+13 days'), NULL, 'active'),
          (8, 6, 3, date('now', '-5 days'),  date('now', '+9 days'),  NULL, 'active'),
          (9, 9, 4, date('now', '-6 days'),  date('now', '+8 days'),  NULL, 'active'),
          (10, 11, 2, date('now', '-2 days'), date('now', '+12 days'), NULL, 'active'),
          (11, 2, 1, date('now', '-40 days'), date('now', '-26 days'), date('now', '-27 days'), 'returned'),
          (12, 5, 2, date('now', '-35 days'), date('now', '-21 days'), date('now', '-20 days'), 'returned'),
          (13, 7, 3, date('now', '-30 days'), date('now', '-16 days'), date('now', '-18 days'), 'returned');

        -- 5. Audit Log Seed
        INSERT OR IGNORE INTO audit_logs (id, user_id, action, entity_type, entity_id, details)
        VALUES
          (1, 1, 'SYSTEM_INIT', 'system', 0, 'Schema migration and baseline system init complete'),
          (2, 1, 'CATALOG_SEEDED', 'books', 12, 'Preloaded 12 canonical software engineering titles'),
          (3, 2, 'LOAN_CREATED', 'loans', 1, 'Issued Clean Code to member LIB-2026-1001'),
          (4, 2, 'LOAN_CREATED', 'loans', 2, 'Issued Clean Code to member LIB-2026-1002');
        "#,
    },
];

/// Run all pending migrations idempotently inside an IMMEDIATE transaction
pub fn run_migrations(conn: &mut Connection) -> Result<(), DbError> {
    // 1. Ensure migrations metadata table exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version INTEGER NOT NULL UNIQUE,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        )",
        [],
    )?;

    // 2. Query applied migrations
    let applied_versions: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT version FROM _migrations ORDER BY version ASC")?;
        let rows = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<i64>, _>>()?;
        rows
    };

    // 3. Apply missing migrations in strict order
    for migration in MIGRATIONS {
        if !applied_versions.contains(&migration.version) {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tx.execute_batch(migration.sql)?;
            tx.execute(
                "INSERT INTO _migrations (version, name) VALUES (?1, ?2)",
                params![migration.version, migration.name],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}
