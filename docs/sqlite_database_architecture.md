# SQLite Database Architecture & Concurrency Specification

**Application:** Professional Library Management System (Topcoat + HTMX + Tailwind)  
**Storage Engine:** SQLite 3 in WAL Mode (`rusqlite` + `r2d2` + `tokio::task::spawn_blocking`)  
**Implementation Source:**
- Connection Pool & PRAGMAs: [`src/db/mod.rs`](../src/db/mod.rs)
- Relational Schema & Seeds: [`db/schema.sql`](../db/schema.sql) and [`db/seeds.sql`](../db/seeds.sql)
- Migrations: [`src/db/migrations.rs`](../src/db/migrations.rs)
- Domain Models: [`src/db/models.rs`](../src/db/models.rs)
- Atomic Circulation Transactions: [`src/db/circulation.rs`](../src/db/circulation.rs)
- Indexed Queries: [`src/db/queries.rs`](../src/db/queries.rs)

---

## 1. Architecture Overview & Performance Principles

In a server-rendered Topcoat application, SQLite runs in-process with zero network overhead. With Write-Ahead Logging (WAL) and memory mapping, SQLite acts as an ultra-fast, high-throughput in-process relational store:

```mermaid
graph TD
    subgraph "Tokio Async Web Runtime (Topcoat)"
        H1["HTTP Handler: Search Catalog"] -->|spawn_blocking| R1["Reader Connection (Thread A)"]
        H2["HTTP Handler: Member Lookup"] -->|spawn_blocking| R2["Reader Connection (Thread B)"]
        H3["HTTP Handler: Issue Loan"] -->|spawn_blocking| W1["Writer Connection (Thread C)"]
    end

    subgraph "SQLite Storage Engine (WAL Mode)"
        R1 -.->|Zero-Lock Read| DB["library.db (Mmap 256MB)"]
        R2 -.->|Zero-Lock Read| DB
        R1 -.->|Read Committed| WAL["library.db-wal (Shared Ringbuffer)"]
        W1 ==>|BEGIN IMMEDIATE Lock| WAL
        WAL -.->|Auto-checkpoint 1000 pages| DB
    end
```

### Core Engine Principles
1. **Zero-Lock Concurrent Reads**: Readers never block writers, and writers never block readers. Any number of HTTP requests can search books and members simultaneously while a librarian commits a loan.
2. **Deterministic Single Writer**: SQLite permits only one write transaction at any microsecond. By enforcing `BEGIN IMMEDIATE` transactions, we eliminate the infamous `SQLITE_BUSY` deadlock during deferred lock upgrades.
3. **RAM-Resident Index Traversal**: Through `PRAGMA cache_size = -64000` (64MB) and `PRAGMA mmap_size = 268435456` (256MB), the library catalog B-tree resides directly in OS memory-mapped pages, reducing query latencies to sub-millisecond range.

---

## 2. Tuning Pragmas

Every SQLite connection checked out from the pool executes the following PRAGMAs:

| Pragma Directive | Setting | Performance & Durability Impact |
| :--- | :--- | :--- |
| `PRAGMA journal_mode` | `WAL` | Enables Write-Ahead Logging. Readers use snapshots while writers append to the `-wal` file. |
| `PRAGMA synchronous` | `NORMAL` | Syncs the WAL to disk at checkpoints rather than on every commit. Boosts write throughput by 10x-50x while preserving full crash durability against power loss. |
| `PRAGMA foreign_keys` | `ON` | Enforces referential integrity at the database core (prevents orphaned loans or illegal record deletions). *Set per connection.* |
| `PRAGMA busy_timeout` | `5000` | SQLite sleeps and retries automatically for up to 5,000ms when waiting for a write lock, eliminating immediate busy lock errors. |
| `PRAGMA cache_size` | `-64000` | Allocates 64MB of RAM page cache per connection. Negative values denote KiB. |
| `PRAGMA mmap_size` | `268435456` | Enables 256MB of memory-mapped I/O, allowing reads to bypass OS `read()` syscalls. |
| `PRAGMA temp_store` | `MEMORY` | Forces temporary index tables, sorting buffers, and aggregations to reside in RAM. |
| `PRAGMA wal_autocheckpoint`| `1000` | Automatically flushes WAL log pages to the main database file every 1,000 pages (~4MB). |

---

## 3. Relational Entity Relationships

```mermaid
erDiagram
    USERS ||--o{ AUDIT_LOGS : performs
    USERS {
        INTEGER id PK
        TEXT username UK
        TEXT role
    }

    MEMBERS ||--o{ LOANS : borrows
    MEMBERS {
        INTEGER id PK
        TEXT membership_number UK
        TEXT name
        TEXT email UK
        TEXT status
    }

    BOOKS ||--o{ LOANS : contains
    BOOKS {
        INTEGER id PK
        TEXT isbn UK
        TEXT title
        TEXT author
        INTEGER total_copies
        INTEGER available_copies
    }

    LOANS {
        INTEGER id PK
        INTEGER book_id FK
        INTEGER member_id FK
        TEXT loan_date
        TEXT due_date
        TEXT return_date
        TEXT status
    }

    AUDIT_LOGS {
        INTEGER id PK
        INTEGER user_id FK
        TEXT action
        TEXT entity_type
        INTEGER entity_id
        TEXT details
    }
```

*For complete column definitions and CHECK constraints, see [`db/schema.sql`](../db/schema.sql).*

---

## 4. Sub-Millisecond Indexing Strategy

To guarantee immediate responses for HTMX search inputs and circulation dashboards, targeted indexes eliminate full-table scans:

- **Books Catalog**:
  - `idx_books_isbn` (Unique): Instant ISBN-10/13 point lookups.
  - `idx_books_title`, `idx_books_author` (`COLLATE NOCASE`): Real-time case-insensitive prefix/substring searches.
  - `idx_books_category_title`: Composite index for category filtering and sorting.
  - `idx_books_available` (Partial): Index on `available_copies WHERE available_copies > 0` for fast checkout combobox searches.
- **Patrons & Members**:
  - `idx_members_membership_number`, `idx_members_email` (Unique): Fast barcode and email resolution.
  - `idx_members_name` (`COLLATE NOCASE`): Name-based autocomplete search.
- **Circulation Desk**:
  - `idx_loans_active_due_date` (Partial): `(status, due_date) WHERE status != 'returned'`. Filters out closed loans (~90% of table), keeping active/overdue checks cached in minimal memory.
  - `idx_loans_member_status`, `idx_loans_book_status`: Composite indexes for member drawer lookups and deletion guards.

---

## 5. ACID Concurrency Control (Zero Race Conditions)

To prevent double-lending race conditions when multiple librarians attempt to check out the final available copy of a title simultaneously:

```mermaid
sequenceDiagram
    autonumber
    actor Librarian A
    actor Librarian B
    participant DB as SQLite Engine (WAL)

    Note over DB: Book has available_copies = 1
    Librarian A->>DB: BEGIN IMMEDIATE
    DB-->>Librarian A: Reserved Lock Granted
    Librarian B->>DB: BEGIN IMMEDIATE
    Note over Librarian B: SQLite sleeps (busy_timeout = 5000ms)

    Librarian A->>DB: UPDATE books SET available_copies = available_copies - 1 WHERE id = 1 AND available_copies > 0
    DB-->>Librarian A: rows_affected = 1 (SUCCESS)
    Librarian A->>DB: INSERT INTO loans ...
    Librarian A->>DB: COMMIT (Releases Lock)

    DB-->>Librarian B: Reserved Lock Granted (Woken from sleep)
    Librarian B->>DB: UPDATE books SET available_copies = available_copies - 1 WHERE id = 1 AND available_copies > 0
    DB-->>Librarian B: rows_affected = 0 (INVENTORY EXHAUSTED)
    Librarian B->>DB: ROLLBACK
    Note over Librarian B: Returns Err(DbError::InventoryExhausted) to UI
```

*Implementation reference: [`issue_loan_tx`](../src/db/circulation.rs) and [`return_book_tx`](../src/db/circulation.rs).*
