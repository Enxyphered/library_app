//! Atomic Circulation Transactions (Lending & Returning)
//!
//! Concurrency Guarantees:
//! 1. Uses `BEGIN IMMEDIATE` to prevent SQLite lock-upgrade deadlocks.
//! 2. Double-Lending Race Condition Prevention:
//!    Atomic decrement with strict WHERE condition:
//!    `UPDATE books SET available_copies = available_copies - 1 WHERE id = ? AND available_copies > 0;`
//!    If two requests hit the server concurrently for the final copy, exactly one succeeds (changes() == 1)
//!    and the other receives changes() == 0, triggering an immediate rollback with `InventoryExhausted`.
//! 3. Relational Safety: Database-level CHECK constraint (`available_copies >= 0 AND available_copies <= total_copies`)
//!    guarantees that inventory numbers never corrupt under any execution path.

use crate::db::models::{Loan, LoanStatus, MemberStatus};
use crate::db::DbError;
use rusqlite::{params, Connection, Row, Transaction};

/// Convert a Rusqlite row into a Loan struct
pub fn map_loan_row(row: &Row) -> rusqlite::Result<Loan> {
    let status_str: String = row.get(6)?;
    let status = LoanStatus::try_from(status_str.as_str())
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

    Ok(Loan {
        id: row.get(0)?,
        book_id: row.get(1)?,
        member_id: row.get(2)?,
        loan_date: row.get(3)?,
        due_date: row.get(4)?,
        return_date: row.get(5)?,
        status,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

/// Atomically issues a loan with strict concurrency and member eligibility checks.
///
/// Execution Sequence (ACID Transaction):
/// 1. Member Eligibility Verification: Validates patron exists and is not suspended.
/// 2. Atomic Decrement Guard: Decrements book inventory iff available_copies > 0.
/// 3. Zero-Stock Guard: Checks affected row count; aborts if inventory was 0.
/// 4. Loan Record Creation: Inserts new active loan record with calculated due date.
/// 5. Audit Logging: Records transaction details for security audit trail.
pub fn issue_loan_tx(
    tx: &Transaction,
    book_id: i64,
    member_id: i64,
    librarian_user_id: Option<i64>,
    duration_days: i32,
) -> Result<Loan, DbError> {
    // Step 1: Verify Member exists and is Active
    let member_status_str: Result<String, rusqlite::Error> = tx.query_row(
        "SELECT status FROM members WHERE id = ?1",
        params![member_id],
        |r| r.get(0),
    );

    let member_status_str = match member_status_str {
        Ok(s) => s,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(DbError::NotFound {
                entity: "Member",
                id: member_id.to_string(),
            });
        }
        Err(e) => return Err(DbError::Sqlite(e)),
    };

    let member_status = MemberStatus::try_from(member_status_str.as_str())
        .map_err(|e| DbError::ConstraintViolation(e))?;

    if member_status == MemberStatus::Suspended {
        return Err(DbError::MemberSuspended { member_id });
    }

    // Step 2: Atomic Decrement with Race Guard
    // If available_copies == 0, this UPDATE affects 0 rows.
    let rows_updated = tx.execute(
        "UPDATE books
         SET available_copies = available_copies - 1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1 AND available_copies > 0",
        params![book_id],
    )?;

    if rows_updated == 0 {
        // Either the book does not exist or all copies are already loaned out.
        // Check if book exists to provide the exact error
        let book_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM books WHERE id = ?1)",
                params![book_id],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if !book_exists {
            return Err(DbError::NotFound {
                entity: "Book",
                id: book_id.to_string(),
            });
        } else {
            return Err(DbError::InventoryExhausted { book_id });
        }
    }

    // Step 3: Insert Loan Record
    let due_date_modifier = format!("+{} days", duration_days.max(1));
    tx.execute(
        "INSERT INTO loans (book_id, member_id, loan_date, due_date, status)
         VALUES (
             ?1,
             ?2,
             strftime('%Y-%m-%d', 'now'),
             strftime('%Y-%m-%d', 'now', ?3),
             'active'
         )",
        params![book_id, member_id, due_date_modifier],
    )?;

    let loan_id = tx.last_insert_rowid();

    // Step 4: Write Audit Log Entry
    let audit_details = serde_json::json!({
        "action": "ISSUE_LOAN",
        "book_id": book_id,
        "member_id": member_id,
        "loan_id": loan_id,
        "duration_days": duration_days
    }).to_string();

    tx.execute(
        "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, details)
         VALUES (?1, 'LOAN_CREATED', 'loans', ?2, ?3)",
        params![librarian_user_id, loan_id, audit_details],
    )?;

    // Step 5: Return complete populated Loan entity
    let loan = tx.query_row(
        "SELECT id, book_id, member_id, loan_date, due_date, return_date, status, created_at, updated_at
         FROM loans WHERE id = ?1",
        params![loan_id],
        map_loan_row,
    )?;

    Ok(loan)
}

/// Atomically returns a book, restoring catalog inventory and marking the loan closed.
///
/// Execution Sequence (ACID Transaction):
/// 1. Loan Verification & Transition: Updates loan status to 'returned' WHERE status IN ('active', 'overdue').
/// 2. Double-Return Guard: If 0 rows modified, rejects as already returned or non-existent.
/// 3. Atomic Increment: Increments book's available_copies, bounded by total_copies CHECK constraint.
/// 4. Audit Logging: Records return timestamp and actor.
pub fn return_book_tx(
    tx: &Transaction,
    loan_id: i64,
    librarian_user_id: Option<i64>,
) -> Result<Loan, DbError> {
    // Step 1: Update Loan Status to 'returned' with conditional guard
    let rows_updated = tx.execute(
        "UPDATE loans
         SET status = 'returned',
             return_date = strftime('%Y-%m-%d', 'now'),
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1 AND status IN ('active', 'overdue')",
        params![loan_id],
    )?;

    if rows_updated == 0 {
        // Verify if loan exists at all
        let loan_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM loans WHERE id = ?1)",
                params![loan_id],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if !loan_exists {
            return Err(DbError::NotFound {
                entity: "Loan",
                id: loan_id.to_string(),
            });
        } else {
            return Err(DbError::LoanAlreadyReturned { loan_id });
        }
    }

    // Step 2: Increment Book's Available Copies
    // Target book is identified directly from the loan record
    let book_updated = tx.execute(
        "UPDATE books
         SET available_copies = available_copies + 1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = (SELECT book_id FROM loans WHERE id = ?1)",
        params![loan_id],
    )?;

    if book_updated == 0 {
        return Err(DbError::ConstraintViolation(
            "Associated book record could not be updated during return".into(),
        ));
    }

    // Step 3: Record Audit Log
    let audit_details = serde_json::json!({
        "action": "RETURN_BOOK",
        "loan_id": loan_id
    }).to_string();

    tx.execute(
        "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, details)
         VALUES (?1, 'BOOK_RETURNED', 'loans', ?2, ?3)",
        params![librarian_user_id, loan_id, audit_details],
    )?;

    // Step 4: Return updated Loan struct
    let loan = tx.query_row(
        "SELECT id, book_id, member_id, loan_date, due_date, return_date, status, created_at, updated_at
         FROM loans WHERE id = ?1",
        params![loan_id],
        map_loan_row,
    )?;

    Ok(loan)
}

/// Standalone synchronous helper for lending when a Connection is already available
#[allow(dead_code)]
pub fn issue_loan(
    conn: &mut Connection,
    book_id: i64,
    member_id: i64,
    librarian_user_id: Option<i64>,
    duration_days: i32,
) -> Result<Loan, DbError> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let loan = issue_loan_tx(&tx, book_id, member_id, librarian_user_id, duration_days)?;
    tx.commit()?;
    Ok(loan)
}

/// Standalone synchronous helper for returning when a Connection is already available
#[allow(dead_code)]
pub fn return_book(
    conn: &mut Connection,
    loan_id: i64,
    librarian_user_id: Option<i64>,
) -> Result<Loan, DbError> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let loan = return_book_tx(&tx, loan_id, librarian_user_id)?;
    tx.commit()?;
    Ok(loan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;

    #[test]
    fn test_atomic_circulation_flow() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();

        // Check initial book copies for ID 1
        let (avail, _total): (i32, i32) = conn
            .query_row(
                "SELECT available_copies, total_copies FROM books WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(avail > 0);

        // Issue loan
        let loan = issue_loan(&mut conn, 1, 1, Some(1), 14).unwrap();
        assert_eq!(loan.status, LoanStatus::Active);

        // Verify available copies decreased by 1
        let avail_after: i32 = conn
            .query_row(
                "SELECT available_copies FROM books WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(avail_after, avail - 1);

        // Return book
        let returned = return_book(&mut conn, loan.id, Some(1)).unwrap();
        assert_eq!(returned.status, LoanStatus::Returned);
        assert!(returned.return_date.is_some());

        // Verify available copies restored
        let avail_restored: i32 = conn
            .query_row(
                "SELECT available_copies FROM books WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(avail_restored, avail);
    }

    #[test]
    fn test_exhausted_inventory_prevents_checkout() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();

        // Set available copies to 0
        conn.execute("UPDATE books SET available_copies = 0 WHERE id = 1", [])
            .unwrap();

        // Attempting to loan should fail with InventoryExhausted
        let res = issue_loan(&mut conn, 1, 1, Some(1), 14);
        match res {
            Err(DbError::InventoryExhausted { .. }) => {}
            other => panic!("Expected InventoryExhausted error, got: {:?}", other),
        }
    }
}
