# Vector Iconography System Specification & Component Catalog

**Application:** Topcoat + HTMX + Tailwind CSS (shadcn tokens) Library Management System  
**Implementation Source:** [`src/components/icons.rs`](../src/components/icons.rs) & [`src/components/mod.rs`](../src/components/mod.rs)  
**Engineering Discipline:** Vector Iconography Specialist & Design Systems Engineer  

---

## 1. Executive Summary & Design Mandates

The Library Management System is an operational, high-throughput utility for librarians and patrons. Interface legibility, rapid cognitive triage, and visual reliability are non-negotiable.

### Strict Anti-Emoji & Anti-Unicode Mandate
> [!IMPORTANT]
> **Zero Tolerance for Emojis, Unicode Symbols, or Text Approximations:**
> - NO Unicode glyphs (e.g. `📚`, `👤`, `🔍`, `⚠️`, `⚙️`, `🔑`, `❌`, `✅`, `🏷️`).
> - NO text-character approximations (e.g. `"x"` for close, `"+"` for add, `"v"` or `">"` for chevrons, `"<->"` for circulation).
> - Every single visual cue across navigation, tables, search bars, modals, drawers, status badges, and toast notifications is an **inline SVG vector icon** adhering to modern Lucide / Heroicons engineering specifications.

---

## 2. Geometric & Design Specifications

All icons are designed on a unified, high-precision vector grid:

| Attribute | Standard Value | Rationale |
| :--- | :--- | :--- |
| **Grid & ViewBox** | `viewBox="0 0 24 24"` | Standard 24px coordinate space with 2px padding ring preventing subpixel clipping. |
| **Fill & Stroke** | `fill="none"` / `stroke="currentColor"` | Colors dynamically inherit the surrounding text/semantic Tailwind token (`text-foreground`, `text-primary`, `text-destructive`, etc.). |
| **Stroke Geometry** | `stroke-width="2"` / `stroke-linecap="round"` / `stroke-linejoin="round"` | Consistent stroke weight across all visual glyphs, matching shadcn/ui and Lucide design tokens. |
| **Defensive Layout Sizing** | Default `size-4 shrink-0` (16px) or `size-5 shrink-0` (20px) | Strict `shrink-0` injected into every icon prevents flexbox layout collapse in tight table cells or compact buttons. |
| **Accessibility (A11y)** | `aria-hidden="true"` by default; `role="img"` + `aria-label` when `label` supplied | Screen readers skip decorative icons; standalone icons announce their semantic intent. |

---

## 3. Topcoat Component Architecture

All icons are organized in [`src/components/icons.rs`](../src/components/icons.rs) as reusable Topcoat `#[component]` functions with dual naming conventions:

1. **Idiomatic Rust Snake Case:** `icon_book_open`, `icon_search`, `icon_dashboard`, `icon_spinner`.
2. **Component PascalCase Aliases:** `IconBookOpen`, `IconSearch`, `IconDashboard`, `IconSpinner`.

### Component Signature Blueprint

```rust
#[component]
pub async fn icon_name(
    /// Tailwind sizing & styling classes. Defaults to "size-4" with defensive "shrink-0".
    #[default("size-4")] class: &'static str,
    /// Accessible label announced to screen readers. When omitted, icon is aria-hidden="true".
    #[default] #[into] label: String,
    /// Forwarded HTML/SVG attributes (e.g. id, hx-* attributes, style).
    #[default] mut attrs: Attributes,
) -> Result<impl View>
```

---

## 4. Comprehensive Icon Inventory & Catalog

### 4.1 Navigation & Identity

| Component Name | PascalCase Alias | Geometry / Description | Primary Usage Context |
| :--- | :--- | :--- | :--- |
| `icon_library` | `IconLibrary` | 4 Book spines in stack (`m16 6 4 14`, `M12 6v14`, `M8 8v12`, `M4 4v16`, `M2 20h20`) | Application Brand Crest, Sidebar Header, Library System identity |
| `icon_book_open` | `IconBookOpen` | Open spread volume with left/right pages | Catalog navigation pill, Book records, Active lending tables |
| `icon_book` | `IconBook` | Single bound hardcover volume with spine | Inventory item detail, copy count badges |
| `icon_dashboard` | `IconDashboard` / `IconLayoutDashboard` | 4-quadrant asymmetric card layout | Dashboard navigation, metric summary overviews |
| `icon_members` | `IconMembers` / `IconUsers` | Multi-person silhouette with foreground leader | Members Directory navigation, patron lookup drawer |
| `icon_circulation`| `IconCirculation` / `IconArrowLeftRight` | Bidirectional horizontal swap arrows (`M20 7H4`, `M4 17h16`) | Circulation Desk, Issue Loan / Return desk |
| `icon_repeat` | `IconRepeat` | Clockwise dual-arrow circulation loop | Loan renewals, recurring borrowing history |
| `icon_sun` | `IconSun` | Solar core with 8 radial ray points | Light theme active indicator / Theme toggle button |
| `icon_moon` | `IconMoon` | Geometric crescent moon arc | Dark theme active indicator / Theme toggle button |
| `icon_settings` | `IconSettings` | 8-tooth mechanical cog with central bore | System preferences, database PRAGMA telemetry |

### 4.2 Authentication & Session

| Component Name | PascalCase Alias | Geometry / Description | Primary Usage Context |
| :--- | :--- | :--- | :--- |
| `icon_lock` | `IconLock` | Padlock body with arched shackle (`rect 18x11`, `M7 11V7a5 5 0 0 1 10 0v4`) | Login form password input, session gate status |
| `icon_shield_check`| `IconShieldCheck` | Pointed crest shield containing verified checkmark | Admin role indicator, security audit logs |
| `icon_user` | `IconUser` | Single person silhouette with circular head | Librarian avatar badge, patron profile cards |
| `icon_user_check` | `IconUserCheck` | Person silhouette with verified checkmark | Active member badge, member good standing confirmation |
| `icon_key` | `IconKey` | Circular bow with notched bit stem | API authorization tokens, session cookie credentials |
| `icon_log_in` | `IconLogIn` | Arrow directing into open bracket gate | Unauthenticated session trigger, sign-in CTA |
| `icon_log_out` | `IconLogOut` | Arrow directing out of open bracket gate | Session termination button, topbar logout trigger |

### 4.3 Operational & Action Controls

| Component Name | PascalCase Alias | Geometry / Description | Primary Usage Context |
| :--- | :--- | :--- | :--- |
| `icon_search` | `IconSearch` | Magnifying lens with 45-degree handle | Live debounced search input adornment (`/books/search`, `/members/search`) |
| `icon_plus` | `IconPlus` | Symmetrical centered crosshairs | "Add Book" modal trigger, "New Loan" quick button |
| `icon_pencil` | `IconPencil` / `IconEdit` | Slanted stylus with graphite tip and eraser band | "Edit Book" modal trigger, edit patron details |
| `icon_trash` | `IconTrash` / `IconDelete` | Waste bin container with lid handle and dual inner ribs | Destructive record deletion trigger, inline confirmation |
| `icon_x` | `IconX` / `IconClose` | Sharp diagonal cross (`M18 6 6 18`, `m6 6 12 12`) | Modal close button, slide-over drawer dismissal, toast dismiss |
| `icon_check` | `IconCheck` | Standard single polyline checkmark (`M20 6 9 17l-5-5`) | Inline confirmation "Yes", successful action indicator |
| `icon_check_circle`| `IconCheckCircle`| Enclosed checkmark inside circular perimeter | Returned loan status pill, positive verification badge |
| `icon_chevron_right`| `IconChevronRight`| Single right-pointing acute angle | Breadcrumb separator, table row inspection trigger |
| `icon_chevron_down`| `IconChevronDown` | Single down-pointing acute angle | Select dropdown indicators, accordion expander |
| `icon_chevron_left`| `IconChevronLeft` | Single left-pointing acute angle | Pagination previous, drawer back navigation |
| `icon_arrow_right` | `IconArrowRight` | Horizontal stem with arrowhead | Fast checkout workflow step forward |
| `icon_refresh` | `IconRefresh` / `IconSync` | Dual curved arrows forming a clockwise circle | Instant table reload, cache purge, out-of-band sync |
| `icon_filter` | `IconFilter` | Wide funnel tapering to vertical spout | Category filter dropdown, circulation status filters |
| `icon_barcode` | `IconBarcode` | Viewfinder framing 4 corner brackets around scanner stripes | Book ISBN scanner trigger, Member card scan input |
| `icon_barcode_lines`| `IconBarcodeLines`| 5 vertical barcode bars of uniform height | ISBN-10/13 table display column, book label previews |

### 4.4 Status & Alerts

| Component Name | PascalCase Alias | Geometry / Description | Primary Usage Context |
| :--- | :--- | :--- | :--- |
| `icon_alert_triangle`| `IconAlertTriangle`| Equilateral caution triangle with exclamation point | Overdue loan table rows (`text-rose-600`), stock exhausted warnings |
| `icon_alert_circle` | `IconAlertCircle` | Circular perimeter enclosing exclamation point | Form field validation errors (`aria-invalid="true"`), warning banners |
| `icon_info` | `IconInfo` | Circular perimeter enclosing centered informational stem | Contextual loan duration policy tooltips, helpful guidance |
| `icon_clock` | `IconClock` | Circular clock face with 12:15 hour/minute hands | Loan duration select, due date timestamps, overdue day counter |
| `icon_calendar` | `IconCalendar` | Calendar page sheet with binder rings and header line | Checkout date picker, return schedule logs |
| `icon_spinner` | `IconSpinner` | High-precision loader arc with `animate-spin shrink-0` | Real-time HTMX loading indicator (`#search-indicator`), drawer submit spinner |

---

## 5. Concrete Code Recipes in Topcoat + HTMX

### Recipe 1: Real-Time Debounced Search Input with Spinner
```rust
view! {
    <div class="relative w-full max-w-md">
        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-muted-foreground">
            icon_search(class: "size-4 text-muted-foreground")
        </div>
        <input
            type="search"
            name="q"
            id="catalog-search"
            placeholder="Search by title, author, or ISBN... (Ctrl+K)"
            class="w-full pl-9 pr-10 py-2 text-sm bg-background border border-input rounded-md focus:outline-none focus:ring-2 focus:ring-ring placeholder:text-muted-foreground"
            hx-get="/books/search"
            hx-trigger="input changed delay:250ms, search"
            hx-target="#catalog-results"
            hx-indicator="#search-spinner"
            autocomplete="off"
        />
        <div id="search-spinner" class="htmx-indicator absolute inset-y-0 right-0 pr-3 flex items-center">
            icon_spinner(class: "size-4 text-primary")
        </div>
    </div>
}
```

### Recipe 2: Overdue Status Badge (WCAG 2.1 AA Compliant)
```rust
view! {
    <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-rose-50 text-rose-700 border border-rose-200 dark:bg-rose-950/40 dark:text-rose-400 dark:border-rose-800 shrink-0">
        icon_alert_triangle(class: "size-3.5 text-rose-600 dark:text-rose-400")
        <span>"Overdue (4 Days)"</span>
    </span>
}
```

### Recipe 3: Slide-Over Drawer Header with Clean Dismissal
```rust
view! {
    <div class="p-6 border-b border-border flex items-center justify-between">
        <div class="flex items-center gap-3">
            <div class="size-9 rounded-lg bg-primary/10 text-primary flex items-center justify-center shrink-0">
                icon_circulation(class: "size-5")
            </div>
            <div>
                <h2 id="drawer-title" class="text-lg font-semibold tracking-tight text-foreground">"Issue Book Loan"</h2>
                <p class="text-xs text-muted-foreground">"Scan or enter Member ID and Book Barcode."</p>
            </div>
        </div>
        <button
            hx-on:click="document.getElementById('loan-drawer').remove()"
            class="text-muted-foreground hover:text-foreground p-1.5 rounded-md hover:bg-muted focus:ring-2 focus:ring-ring transition-colors"
            aria-label="Close drawer"
        >
            icon_x(class: "size-5")
        </button>
    </div>
}
```

### Recipe 4: Out-of-Band Toast Notification with Success Verification
```rust
view! {
    <div id="toast-container" hx-swap-oob="beforeend">
        <div class="flex items-center justify-between p-4 bg-background border border-border rounded-lg shadow-lg text-sm text-foreground animate-in slide-in-from-bottom-3" role="alert">
            <div class="flex items-center gap-2.5">
                icon_check_circle(class: "size-4 text-emerald-600 shrink-0")
                <span>"Book successfully returned to shelf inventory."</span>
            </div>
            <button onclick="this.closest('[role=alert]').remove()" class="ml-4 text-muted-foreground hover:text-foreground p-1">
                icon_x(class: "size-4")
            </button>
        </div>
    </div>
}
```
