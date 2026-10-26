-- ============================================================================
-- Topcoat + HTMX Library Application: Seed Data
-- Idempotent initial dataset: Admin/Librarian accounts, Books, Members, Loans
-- ============================================================================

-- Users: Default administrator ('admin' / 'admin') and librarian ('librarian' / 'admin')
-- Hashed using standard Argon2id format matching src/db/migrations.rs
INSERT OR IGNORE INTO users (id, username, password_hash, role)
VALUES 
  (1, 'admin', '$argon2id$v=19$m=19456,t=2,p=1$temUD3doq8QhVP1FKonI+g$YGpEM39YVkqjnGRvC1NFHVjXLiVl/bdx8/17JVS6O7A', 'admin'),
  (2, 'librarian', '$argon2id$v=19$m=19456,t=2,p=1$temUD3doq8QhVP1FKonI+g$YGpEM39YVkqjnGRvC1NFHVjXLiVl/bdx8/17JVS6O7A', 'librarian');

-- Books: 12 standard catalog titles covering classic and modern systems literature
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
  (12, '978-1492040347', 'Programming Rust: Fast, Safe Systems Development', 'Jim Blandy, Jason Orendorff & Leonora Tindall', 'Programming Languages', 5, 5);

-- Members: Diverse patrons with membership numbers and statuses
INSERT OR IGNORE INTO members (id, membership_number, name, email, phone, status)
VALUES
  (1, 'LIB-2026-1001', 'Elena Rostova', 'elena.rostova@example.com', '+1-555-0192', 'active'),
  (2, 'LIB-2026-1002', 'Marcus Vance', 'marcus.vance@example.com', '+1-555-0144', 'active'),
  (3, 'LIB-2026-1003', 'Dr. Aris Thorne', 'aris.thorne@university.edu', '+1-555-0188', 'active'),
  (4, 'LIB-2026-1004', 'Clara O''Connor', 'clara.oconnor@example.com', '+1-555-0112', 'active'),
  (5, 'LIB-2026-1005', 'Julian Mercer', 'julian.mercer@example.com', '+1-555-0167', 'suspended');

-- Loans: Seed realistic circulation history:
-- Book 1 loaned to Member 1 (Active, due in 7 days)
-- Book 1 loaned to Member 2 (Overdue, due 5 days ago)
-- Book 3 loaned to Member 3 (Active, due in 10 days)
-- Book 3 loaned to Member 4 (Overdue, due 3 days ago)
-- Book 4 loaned to Member 1 (Active, due in 14 days)
-- Book 4 loaned to Member 2 (Active, due in 12 days)
-- Book 4 loaned to Member 4 (Active, due in 10 days) -> total copies 5, available 2
-- Book 6 loaned to Member 3 (Active, due in 4 days)
-- Book 9 loaned to Member 4 (Active, due in 8 days)
-- Book 11 loaned to Member 2 (Active, due in 14 days)
-- Also historical returned loans for circulation statistics
INSERT OR IGNORE INTO loans (id, book_id, member_id, loan_date, due_date, return_date, status)
VALUES
  -- Active and Overdue Loans
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
  
  -- Historical Returned Loans
  (11, 2, 1, date('now', '-40 days'), date('now', '-26 days'), date('now', '-27 days'), 'returned'),
  (12, 5, 2, date('now', '-35 days'), date('now', '-21 days'), date('now', '-20 days'), 'returned'),
  (13, 7, 3, date('now', '-30 days'), date('now', '-16 days'), date('now', '-18 days'), 'returned');

-- Initial Activity Log entries
INSERT OR IGNORE INTO audit_logs (id, user_id, action, entity_type, entity_id, details)
VALUES
  (1, 1, 'SYSTEM_INIT', 'system', 0, 'Initial database schema and seeds created'),
  (2, 1, 'CATALOG_SEEDED', 'books', 12, 'Catalog populated with 12 foundational systems titles'),
  (3, 2, 'LOAN_CREATED', 'loans', 1, 'Issued "Clean Code" to member LIB-2026-1001'),
  (4, 2, 'LOAN_CREATED', 'loans', 2, 'Issued "Clean Code" to member LIB-2026-1002');
