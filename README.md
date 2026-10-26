# Athenaeum — Library Management Application

A high-performance, server-rendered library management application built with **Rust**, **[Topcoat](https://github.com/topcoat-rs/topcoat)**, **HTMX**, **Tailwind CSS v4** (shadcn design tokens), and **SQLite** (WAL mode).

---

## Architecture Highlights

- **Pure Server-Rendered Views**: Built using Topcoat's native `view!` template macro and modular `#[component]` architecture. Zero client-side JavaScript hydration frameworks or WebSocket overhead (`zero #[shard]`).
- **Seamless Hypermedia (HTMX)**: Dynamic catalog search, debounced filtering, out-of-band updates (`hx-swap-oob`), slide-over drawers, and modals.
- **ACID Circulation Concurrency**: Safe concurrent lending using SQLite `BEGIN IMMEDIATE` transactions with atomic decrement guards (`WHERE available_copies > 0`) preventing double-lending race conditions.
- **Design Tokens**: Styled with Tailwind CSS v4 using semantic design tokens (`bg-background`, `text-foreground`, `border-border`, `bg-card`, `bg-primary`, etc.) with full dark/light theme support and responsive off-canvas mobile navigation.
- **Zero-Flicker Stateless Auth**: JWT stored in `HttpOnly`, `SameSite=Lax` browser cookies with zero client flash.

---

## Prerequisites

1. **Rust Toolchain**: Rust 2024 edition compatible compiler (Rust 1.85+ recommended).
   ```bash
   rustup update
   ```
2. **Topcoat CLI**:
   ```bash
   cargo install --git https://github.com/topcoat-rs/topcoat.git --tag v0.11.0 topcoat-cli --locked
   ```

---

## Setup & Getting Started

### 1. Clone the Repository
```bash
git clone https://github.com/Enxyphered/library_app.git
cd library_app
```

### 2. Clone the Topcoat Framework into `topcoat/`
The project's `Cargo.toml` uses local path dependencies to Topcoat crates (`topcoat/crates/...`). Cloning Topcoat directly into `topcoat/` also provides AI coding assistants with immediate, indexed access to Topcoat's internal APIs, macro signatures, and documentation.

```bash
git clone https://github.com/topcoat-rs/topcoat.git topcoat
```

### 3. Bundle Assets
Topcoat scans templates and compiles required Tailwind utility classes into content-hashed static assets:
```bash
topcoat asset bundle
```

### 4. Run the Development Server
```bash
topcoat dev
```
Open **`http://127.0.0.1:3000`** in your browser.

---

## Default Credentials

The database initializes automatically on first run (`library.db`) with idempotent migrations and baseline seed data:

| Username | Password | Role | Access |
| :--- | :--- | :--- | :--- |
| **`admin`** | `admin` | Administrator | Full catalog, members, and circulation access |
| **`librarian`** | `admin` | Staff Librarian | Catalog lookups, circulation checkouts, returns |

---

## Project Structure

```text
├── db/
│   ├── schema.sql           # DDL schema definition (tables, constraints, indexes)
│   └── seeds.sql            # Initial sample seed dataset
├── deployment/
│   ├── DEPLOY_LINUX.md      # Droplet deployment guide and systemd configuration
│   ├── deploy-to-droplet.ps1# 1-click cross-compilation & deploy script (PowerShell)
│   ├── deploy-to-droplet.bat# 1-click cross-compilation & deploy script (CMD)
│   ├── Caddyfile            # Production reverse proxy configuration with TLS & rate limiting
│   └── library_app.service  # Systemd service unit file
├── docs/
│   ├── sqlite_database_architecture.md      # SQLite WAL tuning and concurrency spec
│   ├── core_library_modules_architecture.md # Domain subsystem data flow documentation
│   └── vector_iconography_system.md         # SVG iconography standards and usage
├── examples/
│   └── bundle_linux.rs      # Cross-compilation asset bundler helper
├── src/
│   ├── auth/                # JWT cookie verification and password hashing
│   ├── cache/               # Bounded in-memory cache and entity version invalidators
│   ├── components/          # Reusable UI components (modals, toasts, badges, comboboxes, SVGs)
│   ├── db/                  # SQLite connection pool, migrations, queries, and transactions
│   ├── routes/              # Modular Topcoat route endpoints and view templates
│   │   ├── books/           # Catalog views, CRUD modals, and table filters
│   │   ├── members/         # Member directory, status toggle, and profile drawer
│   │   ├── loans/           # Circulation desk, loan issuance, and return actions
│   │   ├── auth.rs          # Login and logout handlers
│   │   ├── dashboard.rs     # System overview metrics and active circulation table
│   │   └── layout.rs        # Responsive root app shell and sidebar navigation
│   ├── logging.rs           # High-precision HTTP latency and request logging layer
│   └── main.rs              # Application entrypoint and router configuration
├── static/
│   ├── app.js               # Modular vanilla JS controller dispatcher
│   ├── theme-init.js        # Anti-flash dark/light mode bootstrap script
│   └── htmx.min.js          # Bundled HTMX runtime
├── topcoat/                 # (Ignored) Topcoat framework repository
├── build.rs                 # Topcoat Tailwind build configuration
├── Cargo.toml               # Project dependencies and release profiles
└── styles.css               # Tailwind CSS v4 source stylesheet with shadcn tokens
```

---

## Production Deployment

This project supports automated **cross-compilation** from Windows to Linux:
1. It cross-compiles an optimized musl-libc Linux binary locally using `cargo zigbuild`.
2. It bundles static assets and uploads only the ~7 MB binary + assets to the remote server over SSH/SCP.
3. The remote Linux server requires **no Rust toolchains, compilers, or Cargo installed**.

For full instructions, consult [`deployment/DEPLOY_LINUX.md`](deployment/DEPLOY_LINUX.md).

---

## Quality & Testing

```bash
# Verify compilation and linting
cargo check

# Run test suite
cargo test

# Synchronize Tailwind stylesheet and static fingerprints
topcoat asset bundle
```
