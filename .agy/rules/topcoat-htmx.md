---
description: Project rules for Topcoat application development forbidding shards in favor of HTMX and modular data-attribute JS
globs: ["**/*.rs", "**/*.html", "**/*.js", "Cargo.toml"]
---

# Architecture & Interactivity Rules

## 1. Strictly Prohibit Topcoat Shards & Auto-JS
- **NEVER use `#[shard]`**: Do not create or import `shard` components.
- **NEVER use runtime client-side reactive expressions (`$(...)`)**: Do not rely on Topcoat's compiler to transpile Rust expressions into browser JavaScript or WebSocket-synchronized live state.
- Keep Topcoat serving pure server-rendered HTML via `#[page]`, `#[component]`, and `#[layout]`.

## 2. Interactivity via HTMX
- All dynamic UI interactions (filtering, form submissions, inline editing, infinite scroll, tabs, lazy loading) must be driven by **HTMX**.
- Return server-rendered HTML fragments or components directly for `hx-get` and `hx-post` endpoints.
- Leverage standard attributes:
  - `hx-get="..."` / `hx-post="..."`
  - `hx-target="#element-id"`
  - `hx-swap="outerHTML"` (or `innerHTML`, `beforeend`)
  - `hx-indicator="#loading"`

## 3. Minimal Modular JavaScript (`data-*` Attribute Driven)
- Where client-only DOM manipulation is necessary (e.g. modals, custom dropdown toggles, clipboard copy, theme switches):
  - Do NOT write monolithic scripts or inline event handlers.
  - Implement small, decoupled vanilla JS modules initialized by checking `data-*` attributes on the DOM (e.g., `data-controller="tabs"`, `data-action="toggle"`).
  - Serve all client scripts and static assets via Topcoat's native **`asset!`** macro so paths are content-hashed, fingerprinted, and bundled at build time:
    ```rust
    <script src=(topcoat::asset!("static/app.js")) defer></script>
    ```
  - **Mandatory for External Static Assets**: External web URLs for static files (e.g. `topcoat::asset!("https://unpkg.com/htmx.org@2.0.4")` or fonts/vendor scripts) MUST also use `asset!`. Topcoat downloads, caches, content-hashes, and bundles them locally at build time, ensuring zero CDN runtime dependencies and full offline bundle support.
  - For page-specific or feature-specific controllers in `static/controllers/<feature>.js`:
    ```rust
    <script src=(topcoat::asset!("static/controllers/editor.js")) type="module" defer></script>
    ```
  - Support HTMX swap events (`document.addEventListener('htmx:afterSwap', ...)`) so newly swapped DOM nodes automatically bind behavior. [`static/app.js`](static/app.js) already provides this dispatcher.

## 4. Mandatory Styling: Tailwind CSS & Shadcn Design System
- **Strictly Tailwind**: All page styling and UI components must exclusively use Tailwind CSS utilities.
- **Mandatory Asset Bundling on Style Changes**:
  - Whenever styles, utility classes, or static files are modified, **ALWAYS** run `topcoat asset bundle` (or `topcoat asset clean; cargo build; topcoat asset bundle`).
  - Topcoat extracts utility classes from Rust template views and writes the fingerprinted stylesheet to `target/debug/assets/`. Without running this command, new classes will not be available in the browser stylesheet.
- Configure Tailwind via Topcoat's native tooling:
  - `Cargo.toml`: `topcoat = { ..., features = ["tailwind"] }` in both `[dependencies]` and `[build-dependencies]`.
  - `build.rs`: `topcoat::tailwind::BuildConfig::new().render().unwrap();`.
  - Root layout: `<link rel="stylesheet" href=(topcoat::tailwind::stylesheet!())>`.
- **Shadcn Design Tokens (HSL Variables in `styles.css`)**:
  Style all components using shadcn semantic color tokens instead of hardcoded hex values:
  - Backgrounds & Foregrounds: `bg-background text-foreground`, `bg-card text-card-foreground`, `bg-popover text-popover-foreground`.
  - Primary / Secondary: `bg-primary text-primary-foreground`, `bg-secondary text-secondary-foreground`.
  - Muted & Accents: `bg-muted text-muted-foreground`, `hover:bg-accent hover:text-accent-foreground`.
  - Borders & Inputs: `border-border`, `bg-input`, `ring-ring`.
  - Elevation & Shapes: `rounded-md`, `rounded-lg`, `shadow-sm`.

## 5. Shadcn Component Recipes (Pure HTML/HTMX + Minimal Data-Attribute JS)
When implementing UI components, follow these server-rendered recipes. NEVER import React, Radix, or Topcoat Shards:

- **Button (`#[component]`)**:
  - Primary: `inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2`
  - Outline: `border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2`
  - Ghost: `hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2`
  - Destructive: `bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90 h-9 px-4 py-2`

- **Card (`#[component]`)**:
  - Container: `rounded-xl border border-border bg-card text-card-foreground shadow`
  - Header: `flex flex-col space-y-1.5 p-6`
  - Title: `font-semibold leading-none tracking-tight text-lg`
  - Description: `text-sm text-muted-foreground`
  - Content: `p-6 pt-0`
  - Footer: `flex items-center p-6 pt-0`

- **Modal / Dialog**:
  - Use native HTML `<dialog>` with backdrop styling:
    `rounded-lg border border-border bg-background p-6 shadow-lg backdrop:bg-black/50 open:animate-in`
  - Load content dynamically via HTMX:
    `<button hx-get="/modals/edit" hx-target="#modal-container" hx-swap="innerHTML">"Open"</button>`
  - Container script simply invokes `dialog.showModal()` or `dialog.close()` on `data-action="close-modal"`.

- **Dropdown Menu**:
  - Use HTML `<details class="relative" data-controller="dropdown">` with `<summary class="list-none cursor-pointer">`
  - Menu body: `absolute right-0 z-50 mt-2 w-56 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md`
  - Items: `relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none hover:bg-accent hover:text-accent-foreground`

- **Tabs**:
  - HTMX tab swapping: Tab triggers have `hx-get="/tabs/tab-name" hx-target="#tab-content" hx-swap="innerHTML"`.
  - Tab list: `inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground`
  - Tab trigger: `inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium transition-all data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow`

## 6. HyperUI Embedded Component Library
- 560+ pure HTML + Tailwind CSS component templates are locally available in [`docs/hyperui/`](docs/hyperui/).
- Read [`docs/HYPERUI.md`](docs/HYPERUI.md) for the catalogue index.
- When generating tables, filters, stats, inputs, navigation, or side menus:
  1. Inspect the relevant snippet in `docs/hyperui/<section>/<component>/` (e.g. `docs/hyperui/application/tables/1.html`).
  2. Embed the Tailwind classes directly into Topcoat `view! { ... }` macros.
  3. Replace standard actions with `hx-get` / `hx-post` for HTMX interactivity.
  4. Adapt hardcoded colors (e.g. `bg-white`, `border-gray-200`) to the project's shadcn design tokens (`bg-card`, `border-border`, etc.).

## 7. Core Architecture & Defensive Front-End Rules
- **No Inline Scripts in `view! { ... }`**: Never embed multi-line JavaScript directly inside `view!` macro `<script>` tags. Macro templating HTML-escapes logical operators (`&&` becomes `&amp;&amp;`, `<` becomes `&lt;`), throwing `Uncaught SyntaxError` in the browser. Always extract client scripts to `static/*.js` and bundle them using `topcoat::asset!`. Always pass explicit key-value strings for HTML boolean attributes (e.g. `defer="defer"`).
- **Coordinate State Toggles with Framework Utility Engines**: When an element's position, transform, or visibility is styled via Tailwind utility classes (e.g. `translate`, `rotate`, `scale`, `opacity`), JavaScript toggles should add and remove complementary utility classes (e.g. swapping `-translate-x-full` with `translate-x-0`) rather than injecting inline styles or raw CSS overrides. Modern CSS engines treat independent properties (`translate`, `rotate`) backed by `--tw-*` custom variables as higher precedence than legacy `transform` shorthands.
- **Stateless Client Persistence via Cookies**: For client preferences (theme, sidebar collapse state, UI filters) that must persist across page loads and server requests without server-side sessions or WebSockets, store data in browser cookies (`SameSite=Lax; path=/; max-age=...`). Extract them during Topcoat SSR via `topcoat::cookie::cookies(cx).get(...)` to render proper initial markup and prevent client visual flicker.
- **Build Tooling Reactivity**: Ensure `build.rs` declares `println!("cargo:rerun-if-changed=styles.css");` so Tailwind stylesheet regeneration occurs immediately upon editing style definitions.
- **Defensive Layout for Fixed-Dimension Elements in Flexible Containers**: Any visual element with fixed proportions (icons, avatars, brand logos, badges, trigger buttons) placed within a flex or grid parent must have explicit non-shrink defense (`shrink-0`) and minimum dimensions (`min-w-[size] min-h-[size]`). Never apply `overflow: hidden` on shared flex parent containers; confine clipping and truncation (`truncate`) strictly to the fluid text leaf elements.

## 8. Code Standards & Formatting
- Avoid `unsafe` code.
- Format all macro code bodies with `topcoat fmt`.
- Verify with `cargo check` and `cargo test`.
