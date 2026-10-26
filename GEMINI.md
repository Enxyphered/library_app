# Project Instructions & Conventions

This project is built with **Topcoat** (server-rendered Rust web framework), **HTMX**, **Tailwind CSS v4** (shadcn design tokens), and **modular data-attribute driven vanilla JavaScript**.

Read [`AGENTS.md`](AGENTS.md) and [`.agy/rules/topcoat-htmx.md`](.agy/rules/topcoat-htmx.md) for full architecture standards.

---

## 1. Architectural Guardrails: Zero Shards & No WebSockets
- **Strictly No `#[shard]`**: Never create Topcoat reactive shards or use client-side transpiled expressions (`$(...)`).
- **Strictly No Server Sessions or WebSockets**: All client-side persistence (such as dark mode theme, collapsed UI preferences) must use browser cookies (`SameSite=Lax; path=/; max-age=...`). Read cookies in Topcoat via `topcoat::cookie::cookies(cx).get(...)` for flicker-free server rendering.
- **Interactivity via HTMX + Modular JS**: All mutations and server communications must use standard HTMX attributes (`hx-get`, `hx-post`, `hx-target`, `hx-swap`). Client-only behaviors (modals, dropdowns, clipboard, toggles) are implemented as small vanilla JS controllers in `static/` activated via `data-controller="..."` attributes.

---

## 2. Core Front-End Architecture & Defensive Rules

### A. Asset Bundling on Style Changes
- **CRITICAL RULE**: Whenever changes are made to styles, Tailwind classes, or static assets, **ALWAYS** run:
  ```powershell
  topcoat asset bundle
  ```
  *(or `topcoat asset clean; cargo build; topcoat asset bundle` if purging cache)*
- Topcoat scans your Rust source files and compiles the required Tailwind utility classes into the fingerprint-hashed stylesheet inside `target/debug/assets/` referenced by `topcoat::tailwind::stylesheet!()`.
- Failing to run `topcoat asset bundle` after introducing new utility classes (e.g. `w-24`, `w-28`, `min-w-[...]`) causes the browser to fall back to unstyled element defaults, leading to broken alignment and jitter.

### B. No Inline JavaScript in `view! { ... }`
- **Issue**: Topcoat's `view!` template macro HTML-escapes logical operators (`&&` becomes `&amp;&amp;`, `<` becomes `&lt;`), resulting in `Uncaught SyntaxError: Unexpected token ';'` in the browser.
- **Rule**: Never embed multi-line JavaScript directly inside `view!` macro `<script>` tags. Place all scripts in `static/` (e.g. `static/app.js`, `static/theme-init.js`) and link them using Topcoat's content-hashed macro `topcoat::asset!("static/...")`. Always use explicit key-value strings for HTML boolean attributes (e.g. `defer="defer"`).

### C. Coordinate State Toggles with Framework Utility Engines
- **Issue**: In Tailwind CSS v4, transform utilities (e.g. `-translate-x-full`) use independent modern CSS properties (`translate: var(--tw-translate-x) var(--tw-translate-y)`) backed by `@property --tw-*` custom variables. Traditional CSS property overrides like `transform: translateX(0) !important;` will be ignored because the browser's modern `translate` property takes precedence over legacy `transform`.
- **Rule**: Dynamic JavaScript state toggles must manipulate the complementary utility classes directly on the DOM (e.g. removing `-translate-x-full` and adding `translate-x-0`) rather than fighting framework cascades with ad-hoc CSS styles. If writing custom CSS overrides, reset the framework's CSS variables and modern properties explicitly.

### D. Defensive Layout for Fixed-Dimension Elements in Flexible Containers
- **Issue**: Elements with designated physical dimensions (brand logos, icons, action triggers, avatars) get clipped or squished when flex parents shrink or when parent containers apply blanket `overflow-hidden`.
- **Rule**:
  1. Any fixed-dimension element inside Flexbox or Grid layouts must be protected with `shrink-0` (or `flex-shrink: 0`) and explicit minimum sizing constraints (`min-w-[size] min-h-[size]`).
  2. Avoid placing `overflow-hidden` on high-level flex parents that house fixed-dimension visual assets. Confine overflow clipping and text truncation (`truncate`) strictly to the leaf text nodes.

### E. Build Tooling Reactivity
- **Rule**: The build script (`build.rs`) must include `println!("cargo:rerun-if-changed=styles.css");` so that stylesheet updates, theme token modifications, and template classes trigger immediate Tailwind rebuilds without requiring manual Cargo cache busts.

---

## 3. Essential Commands & Verification
- **Asset Bundling**: `topcoat asset bundle` (MUST be executed whenever Tailwind classes, styles.css, or static assets are added or modified)
- **Formatting**: `topcoat fmt` or `cargo fmt`
- **Verification**: `cargo check` and `cargo test`
- **Dev Server**: `topcoat dev` (runs on `http://127.0.0.1:3000`)
