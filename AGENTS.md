# Agent Instructions

Read [`llms.txt`](llms.txt) for Topcoat's core APIs, module routing, and view templating conventions.

## CRITICAL PROJECT ARCHITECTURE DIRECTIVE: NO SHARDS OR AUTO-COMPILED JS

1. **NO `#[shard]`**:
   - Under NO circumstances should you define or use `#[shard]` components or Topcoat's internal WebSocket-backed reactive shard mechanism.
   - Do NOT use Topcoat's compiler to transpile Rust expressions into client-side JavaScript (`$(...)` client event closures or reactive bindings). Keep views as pure server-rendered components (`#[component]`, `#[page]`, `#[layout]`).

2. **Interactivity Model: HTMX + Minimal Modular JS**:
   - **HTMX First**: Use HTMX attributes (`hx-get`, `hx-post`, `hx-target`, `hx-swap`, `hx-trigger`, `hx-indicator`) on standard HTML elements rendered via Topcoat's `view! { ... }` macro.
   - Return modular partial views (rendered with `#[component]` or small route handlers returning HTML fragments) for HTMX replacement targets.
   - **Modular JS**: Client-side JavaScript must be kept minimal, vanilla, and modular.
   - **Data-attribute Activation & `asset!`**: Scripts must be modularized and dynamically activated based on HTML `data-*` attributes (e.g. `data-controller="dropdown"`). Link all scripts (`static/app.js`, `static/controllers/*.js`) using Topcoat's `asset!("static/...")` macro for content-hashed, fingerprinted bundle resolution. Each page or component should only activate the minimal behavior it requires.

3. **Mandatory Asset Bundling on Style Changes**:
   - **CRITICAL RULE**: Whenever changes are made to styles, Tailwind utility classes in templates, or static files, you **MUST** run:
     ```powershell
     topcoat asset bundle
     ```
     *(or `topcoat asset clean; cargo build; topcoat asset bundle` if purging cache)*
   - Topcoat scans your Rust source files and compiles the required Tailwind utility classes into the fingerprint-hashed stylesheet inside `target/debug/assets/` referenced by `topcoat::tailwind::stylesheet!()`.
   - Failing to run `topcoat asset bundle` after introducing new utility classes causes the browser to fall back to unstyled element defaults, leading to layout shifts and broken button sizes.

4. **Essential Topcoat Rust Symbols & Macros Cheat Sheet**:
   Always use these native Topcoat symbols instead of generic/Axum types:
   - **Views & Templating**:
     - `view! { ... }`: The primary server-rendering HTML template macro.
     - `class!("base", "active" if is_active)`: Composes dynamic Tailwind classes without allocating strings or string concats.
     - `attributes! { id="x", ... }`: Spreads attributes into elements via `<div (attrs)>`.
     - `#[component]`: Defines reusable async view components (`async fn my_comp(prop: &str) -> Result<impl View>`).
     - `#[derive(Props)]`: Generates builder prop rules (`#[default]`, `#[into]`) for components.
     - `view::element_id(cx)`: Generates stable, deterministic HTML IDs for accessibility (`for`/`id` linking).
   - **Routing & Handlers**:
     - `module_router!()`: Topcoat's module-based router that maps current module and child `mod` submodules.
     - `#[page]`: Serves GET page requests returning `Result<impl View>`.
     - `#[route(POST)]` (or `#[route(GET)]`, `#[route(DELETE)]`): HTTP route handlers for HTMX mutations/endpoints.
     - `#[layout]`: Defines page wrappers accepting `slot: Slot<'_>`.
     - `href!(handler, ...)`: Type-safe URL generator. Never hardcode internal link strings; pass directly to `href=(href!(my_handler))` or resolve with `.resolve(cx)`.
     - `module_param!(pub item_id: u64)`: Declares URL path parameters for module routing; read using `path_param::<ItemId>(cx)?`.
     - `#[query_params] struct Query`: Memoized query parameter extractor; read using `query_params::<Query>(cx)?`.
     - `router::content::{Form, Json}`: Request body extractors.
   - **Assets & Tailwind**:
     - `topcoat::asset!("static/...")`: Fingerprints, content-hashes, and resolves paths for static assets and scripts at compile time.
     - **External Web Assets Bundling**: Even external static URLs (e.g. CDN scripts, images, external stylesheets: `topcoat::asset!("https://unpkg.com/htmx.org@2.0.4")`) MUST use the `asset!` macro. Topcoat will automatically download, cache, fingerprint, and bundle them locally at build time, preventing external network dependencies in production.
     - `topcoat::tailwind::stylesheet!()`: Links the build-time compiled Tailwind CSS stylesheet in your root layout.
     - `AssetBundle::load()?`: Loads bundled static assets into the router at application startup.
   - **Response Helpers**:
     - `see_other(href!(page).resolve(cx))`: 303 Redirect for POST/Redirect/GET patterns.
     - `ok_or_not_found()?`, `ok_or_forbidden()?`: Extensions on Option/Result via `topcoat::router::error::RouterErrorExt`.

5. **Styling: Mandatory Tailwind CSS & Shadcn Aesthetic**:
   - **Tailwind Only**: All styling must use Tailwind CSS utility classes with shadcn design tokens (`bg-background`, `text-foreground`, `border-border`, `bg-card`, `bg-popover`, `bg-primary`, `bg-muted`, etc.).
   - Setup requires enabling `features = ["tailwind"]` in `Cargo.toml` for both `[dependencies]` and `[build-dependencies]`.
   - The build script (`build.rs`) must invoke `topcoat::tailwind::BuildConfig::new().render().unwrap();`.
   - The root layout must link the compiled stylesheet using `<link rel="stylesheet" href=(topcoat::tailwind::stylesheet!())>`.
   - **HyperUI Catalogue**: 560+ pure HTML + Tailwind component templates are embedded in [`docs/hyperui/`](docs/hyperui/). Consult [`docs/HYPERUI.md`](docs/HYPERUI.md) and adapt their markup and Tailwind structures into Topcoat `view!` components without introducing client-side frameworks.
   - Do NOT write custom inline CSS or ad-hoc style tags. Follow component recipes in `.agy/rules/topcoat-htmx.md` (Buttons, Cards, Dialogs, Dropdowns) using pure server-rendered Topcoat components + HTMX instead of React/Radix.

6. **Formatting, Quality & Robustness**:
   - Unsafe code is strictly disallowed.
   - Format macro bodies with `topcoat fmt` (or `cargo fmt` with Topcoat formatter hooks).
   - Verify code with `cargo check` and `cargo test`.

7. **Core Architecture & Defensive Front-End Rules**:
   - **No Inline JavaScript in `view!`**: Never write multi-line inline JavaScript inside Topcoat's `view! { ... }` macros. The macro HTML-escapes logical operators (`&&` into `&amp;&amp;`, `<` into `&lt;`), resulting in browser `SyntaxError`s. Always place client scripts in `static/` linked via `asset!("static/...")`. Always use explicit key-value strings for HTML boolean attributes (e.g., `defer="defer"`).
   - **Framework Utility Invariance for Dynamic State**: When an element's presentation or visibility is governed by Tailwind utility classes (e.g., `translate`, `rotate`, `scale`, `opacity`, `hidden`), dynamic JavaScript interactions should toggle the complementary framework utility classes (e.g. swapping `-translate-x-full` with `translate-x-0`) rather than fighting framework properties with ad-hoc legacy CSS overrides. Modern CSS separates transforms into independent properties (`translate`, `rotate`) backed by `--tw-*` variables, rendering legacy shorthand overrides ineffective.
   - **Stateless Browser Persistence (Cookies over Sessions/WebSockets)**: When client preferences (theme, UI states, filters) must persist across requests without server sessions or WebSockets, store them in standard browser cookies (`SameSite=Lax; path=/; max-age=...`). Extract them in Topcoat via `cookies(cx).get(...)` to server-render correct initial states, paired with an anti-flash static bootstrap script.
   - **Build Tooling Reactivity**: The build script (`build.rs`) must declare `println!("cargo:rerun-if-changed=styles.css");` so edits to styles and tokens trigger an immediate Tailwind compile without requiring manual Cargo cache busts.
   - **Defensive Layout for Fixed-Dimension Elements in Flexible Containers**: Any element intended to maintain fixed proportions (icons, avatars, brand logos, action buttons, badges) inside Flexbox or Grid layouts must declare non-shrink protection (`shrink-0`) and explicit minimum sizing bounds (`min-w-[size] min-h-[size]`). Never apply `overflow: hidden` to high-level parent containers; isolate truncation strictly to fluid leaf text nodes.
