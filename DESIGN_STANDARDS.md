# Topcoat + HTMX + Tailwind (shadcn tokens) UI/UX Design Standards & Architecture Specification

> **Target Audience:** Component Architects, Frontend Engineers, Full-Stack Developers, and Orchestrator  
> **Application Domain:** Professional Library Management System (Books, Members, Circulation/Lending)  
> **Architecture Pattern:** Server-Rendered Topcoat Components + HTMX Hypermedia Controls + Tailwind CSS with shadcn Design Tokens (Zero heavy client SPA frameworks, zero fragile WebSocket complexity).

---

## 1. Executive Summary & Design System Foundations

The Library Management System is a mission-critical, high-throughput utility for librarians. Librarians deal with continuous inventory queries, rapid member checkouts, and high-frequency return processing. The design philosophy centers on **predictable speed, zero context switching, and resilient server-driven reactivity**.

### 1.1 Core Design Pillars

1. **Information Density with Zero Clutter**: Compact tabular displays, clean typography, strict baseline alignment, and purposeful whitespace. Never force librarians to scroll through excessive margins when scanning 50 titles.
2. **Predictable Hypermedia Flow**: Every HTMX interaction must declare explicit loading states (`hx-indicator`), target surgical DOM updates (`hx-target`), and provide out-of-band telemetry (`hx-swap-oob`) for peripheral metrics without full page refreshes.
3. **Ergonomic Keyboard-First Circulation**: Frequent workflows (e.g., scan ISBN -> verify member -> issue loan) must be achievable without requiring constant mouse navigation.
4. **Resilient Feedback & Safe Actions**: Destructive operations (deleting records, revoking memberships) require in-place two-step server confirmations. Flashes and toast notifications provide instant reassurance and auto-dismiss without interrupting focus.

### 1.2 Design Tokens (shadcn/ui Compatible Semantic Mapping)

All components must strictly use semantic color tokens rather than arbitrary Tailwind hex values to guarantee flawless dark/light theme support and high-contrast WCAG 2.1 AA compliance:

```css
/* Design Token Reference Mapping */
:root {
  --background: 0 0% 100%;           /* #ffffff */
  --foreground: 222.2 84% 4.9%;       /* #020817 */
  --card: 0 0% 100%;
  --card-foreground: 222.2 84% 4.9%;
  --popover: 0 0% 100%;
  --popover-foreground: 222.2 84% 4.9%;
  --primary: 221.2 83.2% 53.3%;      /* Professional Library Slate/Indigo */
  --primary-foreground: 210 40% 98%;
  --secondary: 210 40% 96.1%;
  --secondary-foreground: 222.2 47.4% 11.2%;
  --muted: 210 40% 96.1%;
  --muted-foreground: 215.4 16.3% 46.9%;
  --accent: 210 40% 96.1%;
  --accent-foreground: 222.2 47.4% 11.2%;
  --destructive: 0 84.2% 60.2%;      /* Rose 600 */
  --destructive-foreground: 210 40% 98%;
  --border: 214.3 31.8% 91.4%;
  --input: 214.3 31.8% 91.4%;
  --ring: 221.2 83.2% 53.3%;
  --radius: 0.5rem;
}
```

#### Status Semantic Palette
- **Active / Available**: `bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/40 dark:text-emerald-400 dark:border-emerald-800`
- **Overdue / Critical**: `bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/40 dark:text-rose-400 dark:border-rose-800`
- **Due Soon / Warning**: `bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/40 dark:text-amber-400 dark:border-amber-800`
- **Returned / Archived**: `bg-slate-100 text-slate-700 border-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:border-slate-700`

---

## 2. Information Architecture & Navigation Ergonomics

### 2.1 Global Layout Architecture

1. **Collapsible Sidebar (Desktop: Fixed 256px / Mobile: Off-canvas Drawer)**:
   - Houses primary navigation nodes: `Dashboard`, `Circulation (Loans & Returns)`, `Catalog (Books)`, `Members`, `Settings`.
   - **Active State Rule**: The active route must display an active pill: `bg-primary/10 text-primary font-semibold border-r-2 border-primary` or shadcn `bg-secondary text-foreground font-medium`.
   - **Live Metric Counter Badges**: Badges inside navigation items (e.g., overdue count badge `id="sidebar-overdue-count"`) must carry persistent HTML IDs so any HTMX response in the app can trigger an out-of-band swap (`hx-swap-oob="true"`).
2. **Topbar & Header Context**:
   - Clean breadcrumbs indicating route hierarchy: `Catalog > Computer Science > Distributed Systems`.
   - Rapid circulation quick-buttons: `+ New Loan (F2)` and `+ Add Book (F3)`.
   - Librarian profile pill with active session indicator and clean `hx-post="/logout"` trigger.
3. **Strict Authentication Boundary**:
   - Unauthenticated visitors hitting any route are redirected to `/login` via server 302 or `HX-Redirect: /login`.
   - Login page is cleanly isolated from the admin layout (clean, centered card with zero clutter, branded, autofocus on email/username).

---

## 3. HTMX Interactivity Patterns & Micro-interaction Heuristics

### 3.1 Search-as-You-Type with Defensive UX
When librarians search catalogs or members, real-time feedback is required without overloading the server or creating UI jitter.

#### Rulebook for Realtime Search:
- **Debounce**: Use `delay:250ms` for text fields, and listen for `input changed, search`.
- **Indicators**: Never leave librarians wondering if their keystroke registered. Every search input must link to an explicit spinner via `hx-indicator="#search-spinner"`.
- **Targeting**: Target the `<tbody>` or table container directly (`hx-target="#book-table-body"`), avoiding re-rendering headers or pagination controls unless necessary.
- **URL Synchronization**: Add `hx-push-url="true"` to preserve search query parameters in the browser address bar, enabling reload and bookmarking.
- **Empty States**: Never display a raw empty white block. Return an informative empty state containing an illustration/icon, a clear message ("No books found matching '...'"), and a primary CTA ("Clear filters" or "Add new book").

```html
<!-- Search Input Blueprint -->
<div class="relative w-full max-w-md">
  <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-muted-foreground">
    <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
  </div>
  <input
    type="search"
    name="q"
    id="catalog-search"
    placeholder="Search by title, author, or ISBN... (Ctrl+K)"
    class="w-full pl-9 pr-10 py-2 text-sm bg-background border border-input rounded-md focus:outline-none focus:ring-2 focus:ring-ring focus:border-input transition-colors placeholder:text-muted-foreground"
    hx-get="/books/search"
    hx-trigger="input changed delay:250ms, search"
    hx-target="#catalog-results"
    hx-indicator="#search-indicator"
    hx-push-url="true"
    autocomplete="off"
  />
  <div id="search-indicator" class="htmx-indicator absolute inset-y-0 right-0 pr-3 flex items-center">
    <svg class="animate-spin h-4 w-4 text-primary" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
    </svg>
  </div>
</div>
```

---

### 3.2 Overlay Matrix: Modals vs. Drawers vs. Inline Expansions

To eliminate librarian disorientation, interactions must follow strict contextual hierarchy:

| Interaction Type | Use Case | UX Justification | HTMX Mechanism |
| :--- | :--- | :--- | :--- |
| **Drawer / Sheet (Slide-over)** | **Issue Loan / Fast Checkout** | The librarian needs to reference the catalog list or member details visible on screen while executing the loan transaction. | `hx-get="/loans/new" hx-target="#drawer-container"` |
| **Modal Dialog** | **Add / Edit Book Record** | Focused, self-contained CRUD task requiring clean form validation and modal focus trap. Background is dimmed. | `hx-get="/books/new" hx-target="#modal-container"` |
| **Inline Row Expansion** | **View Copies & Circulation Log** | Quick inspection of individual book copy barcodes or member loan history without opening any overlay. | `hx-get="/books/{id}/copies" hx-target="closest tr" hx-swap="afterend"` |
| **Inline Button Swap** | **Destructive Confirmations** | Prevents disruptive browser popups; converts the "Delete" button directly into a "Confirm?" state. | `hx-get="/books/{id}/delete-confirm" hx-target="this" hx-swap="outerHTML"` |

---

### 3.3 Instant Telemetry with Out-of-Band (`hx-swap-oob`) Updates

When a circulation action occurs (e.g., returning an overdue book), multiple UI components across the screen must synchronize instantaneously without a full reload.

#### Server Response Multi-Payload Example:
```html
<!-- Primary Target: Replaces row in the loans table -->
<tr id="loan-row-42" class="bg-muted/40 transition-colors">
  <td class="py-3 px-4 text-sm font-medium">Clean Code</td>
  <td class="py-3 px-4 text-sm">Sarah Jenkins</td>
  <td class="py-3 px-4 text-sm">
    <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-slate-100 text-slate-700 border border-slate-200">
      Returned
    </span>
  </td>
  <td class="py-3 px-4 text-right text-xs text-muted-foreground">Just now</td>
</tr>

<!-- Out-of-band Swap 1: Update Overdue Counter Badge in Sidebar -->
<span id="sidebar-overdue-count" hx-swap-oob="true" class="ml-auto px-2 py-0.5 text-xs font-bold rounded-full bg-rose-500/15 text-rose-700 border border-rose-200">
  3
</span>

<!-- Out-of-band Swap 2: Append Toast Notification -->
<div id="toast-container" hx-swap-oob="beforeend">
  <div class="toast-item flex items-center justify-between p-4 bg-background border border-border rounded-lg shadow-lg text-sm text-foreground transition-all duration-300 animate-in slide-in-from-bottom-3"
       role="alert">
    <div class="flex items-center gap-2">
      <svg class="h-4 w-4 text-emerald-600 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
      </svg>
      <span>Book <strong>Clean Code</strong> successfully returned.</span>
    </div>
    <button onclick="this.closest('.toast-item').remove()" class="ml-4 text-muted-foreground hover:text-foreground">
      <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
    </button>
  </div>
</div>
```

---

## 4. Accessibility, Status Badges & Destructive Actions

### 4.1 Non-Modal Inline Destructive Confirmation

Standard browser alerts (`window.confirm`) freeze the rendering thread and violate accessibility heuristics. Instead, destructive triggers must swap to an inline confirmation state:

```html
<!-- Initial State -->
<button
  hx-get="/books/104/confirm-delete"
  hx-target="this"
  hx-swap="outerHTML"
  class="px-2.5 py-1 text-xs font-medium text-destructive hover:bg-destructive/10 rounded transition-colors"
  title="Delete book record"
>
  Delete
</button>

<!-- Server Response for /books/104/confirm-delete -->
<div class="inline-flex items-center gap-1.5 p-1 bg-destructive/10 border border-destructive/20 rounded-md">
  <span class="text-xs font-medium text-destructive px-1">Delete?</span>
  <button
    hx-delete="/books/104"
    hx-target="closest tr"
    hx-swap="outerHTML swap:300ms"
    class="px-2 py-0.5 text-xs font-semibold bg-destructive text-destructive-foreground hover:bg-destructive/90 rounded shadow-xs"
  >
    Yes
  </button>
  <button
    hx-get="/books/104/cancel-delete"
    hx-target="closest div"
    hx-swap="outerHTML"
    class="px-2 py-0.5 text-xs font-medium text-muted-foreground hover:text-foreground rounded"
  >
    Cancel
  </button>
</div>
```

### 4.2 High-Contrast Status Badges Specification

Status indicators must pair distinct SVG icons with text to avoid relying solely on color (WCAG Guideline 1.4.1 Use of Color).

| Status | Markup Blueprint | Visual Preview Description |
| :--- | :--- | :--- |
| **Active Loan** | `<span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-50 text-emerald-700 border border-emerald-200/60 dark:bg-emerald-950/40 dark:text-emerald-400 dark:border-emerald-800"><svg class="w-3 h-3 text-emerald-600" fill="currentColor" viewBox="0 0 8 8"><circle cx="4" cy="4" r="3"/></svg> Active</span>` | Green pill with solid dot, clean dark green text |
| **Overdue** | `<span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-medium bg-rose-50 text-rose-700 border border-rose-200/60 dark:bg-rose-950/40 dark:text-rose-400 dark:border-rose-800"><svg class="w-3 h-3 text-rose-600" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg> Overdue</span>` | Rose warning pill with caution triangle icon |
| **Returned** | `<span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-medium bg-slate-100 text-slate-700 border border-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:border-slate-700"><svg class="w-3 h-3 text-slate-500" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/></svg> Returned</span>` | Neutral gray pill with checkmark |
| **Low / Zero Stock** | `<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-mono font-semibold bg-amber-50 text-amber-800 border border-amber-200">0 of 4 Available</span>` | Distinct tabular count indicating availability |

---

## 5. Concrete Component Blueprints

### Blueprint A: Circulation Fast-Checkout Drawer (Slide-Over)

Librarians use this right-side slide-over to process loans without losing sight of the underlying table.

```html
<!-- Slide-Over Drawer Container -->
<div id="loan-drawer" class="fixed inset-0 z-50 overflow-hidden" role="dialog" aria-modal="true" aria-labelledby="drawer-title">
  <!-- Backdrop -->
  <div class="fixed inset-0 bg-background/80 backdrop-blur-sm transition-opacity" 
       hx-on:click="document.getElementById('loan-drawer').remove()"></div>

  <div class="fixed inset-y-0 right-0 pl-10 max-w-full flex">
    <div class="w-screen max-w-md bg-card border-l border-border shadow-2xl flex flex-col">
      
      <!-- Drawer Header -->
      <div class="p-6 border-b border-border flex items-center justify-between">
        <div>
          <h2 id="drawer-title" class="text-lg font-semibold tracking-tight text-foreground">Issue Book Loan</h2>
          <p class="text-xs text-muted-foreground mt-0.5">Scan or enter Member ID and Book Barcode.</p>
        </div>
        <button 
          hx-on:click="document.getElementById('loan-drawer').remove()"
          class="text-muted-foreground hover:text-foreground p-1 rounded-md hover:bg-muted focus:ring-2 focus:ring-ring"
          aria-label="Close drawer"
        >
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
        </button>
      </div>

      <!-- Drawer Body: Fast Form -->
      <form 
        hx-post="/loans" 
        hx-target="#loan-drawer"
        hx-swap="outerHTML"
        class="flex-1 overflow-y-auto p-6 space-y-5"
      >
        <!-- Member Lookup -->
        <div class="space-y-1.5">
          <label for="member-input" class="block text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Member (Barcode / ID / Name) <span class="text-destructive">*</span>
          </label>
          <input 
            type="text" 
            id="member-input" 
            name="member_query" 
            required 
            autofocus
            placeholder="Scan member card or type name..." 
            class="w-full px-3 py-2 text-sm bg-background border border-input rounded-md focus:ring-2 focus:ring-primary focus:border-input"
            hx-get="/members/quick-search"
            hx-trigger="input changed delay:250ms"
            hx-target="#member-search-preview"
          />
          <div id="member-search-preview" class="empty:hidden mt-1 border border-border rounded-md bg-card shadow-xs"></div>
        </div>

        <!-- Book Lookup -->
        <div class="space-y-1.5">
          <label for="book-input" class="block text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Book (ISBN / Barcode) <span class="text-destructive">*</span>
          </label>
          <input 
            type="text" 
            id="book-input" 
            name="book_barcode" 
            required 
            placeholder="Scan book barcode..." 
            class="w-full px-3 py-2 text-sm bg-background border border-input rounded-md font-mono focus:ring-2 focus:ring-primary focus:border-input"
          />
        </div>

        <!-- Loan Period -->
        <div class="space-y-1.5">
          <label for="due-days" class="block text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Loan Duration
          </label>
          <select 
            id="due-days" 
            name="duration_days" 
            class="w-full px-3 py-2 text-sm bg-background border border-input rounded-md focus:ring-2 focus:ring-primary focus:border-input"
          >
            <option value="14" selected>Standard (14 Days)</option>
            <option value="7">Short Loan (7 Days)</option>
            <option value="28">Extended Loan (28 Days)</option>
          </select>
        </div>

        <!-- Submit & Actions -->
        <div class="pt-4 border-t border-border flex items-center justify-end gap-3">
          <button 
            type="button" 
            hx-on:click="document.getElementById('loan-drawer').remove()" 
            class="px-4 py-2 text-sm font-medium text-muted-foreground hover:bg-muted rounded-md transition-colors"
          >
            Cancel
          </button>
          <button 
            type="submit" 
            class="px-4 py-2 text-sm font-semibold bg-primary text-primary-foreground hover:bg-primary/90 rounded-md shadow-sm transition-all focus:ring-2 focus:ring-primary focus:ring-offset-2 flex items-center gap-2"
          >
            <span class="htmx-indicator inline-block"><svg class="animate-spin h-4 w-4" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path></svg></span>
            <span>Issue Loan</span>
          </button>
        </div>
      </form>
    </div>
  </div>
</div>
```

---

### Blueprint B: High-Density Catalog Table with Skeletons & Stock Meter

```html
<div class="w-full overflow-hidden rounded-lg border border-border bg-card shadow-xs">
  <div class="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-border bg-muted/20">
    <div class="flex items-center gap-2">
      <h3 class="text-base font-semibold tracking-tight text-foreground">Book Catalog</h3>
      <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-secondary text-secondary-foreground">248 Total</span>
    </div>
    <div class="flex items-center gap-2">
      <!-- Search filter -->
      <input 
        type="search" 
        name="filter" 
        placeholder="Filter catalog..." 
        hx-get="/books/table" 
        hx-trigger="input changed delay:200ms" 
        hx-target="#catalog-tbody" 
        class="h-9 px-3 text-xs bg-background border border-input rounded-md w-56 focus:ring-1 focus:ring-primary"
      />
      <button 
        hx-get="/books/new-modal" 
        hx-target="#modal-slot" 
        class="h-9 px-3 text-xs font-semibold bg-primary text-primary-foreground rounded-md hover:bg-primary/90 shadow-xs flex items-center gap-1.5"
      >
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
        <span>Add Book</span>
      </button>
    </div>
  </div>

  <div class="overflow-x-auto">
    <table class="w-full text-left border-collapse">
      <thead>
        <tr class="border-b border-border bg-muted/40 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          <th class="py-3 px-4">Title & Author</th>
          <th class="py-3 px-4">ISBN</th>
          <th class="py-3 px-4">Category</th>
          <th class="py-3 px-4">Availability</th>
          <th class="py-3 px-4 text-right">Actions</th>
        </tr>
      </thead>
      <tbody id="catalog-tbody" class="divide-y divide-border text-sm">
        <!-- Typical Data Row -->
        <tr class="hover:bg-muted/30 transition-colors">
          <td class="py-3 px-4 font-medium text-foreground">
            <div>Designing Data-Intensive Applications</div>
            <div class="text-xs text-muted-foreground font-normal">Martin Kleppmann</div>
          </td>
          <td class="py-3 px-4 font-mono text-xs text-muted-foreground">978-1449373320</td>
          <td class="py-3 px-4">
            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-secondary text-secondary-foreground">
              Distributed Systems
            </span>
          </td>
          <td class="py-3 px-4">
            <div class="flex items-center gap-2">
              <div class="w-16 bg-muted rounded-full h-1.5 overflow-hidden">
                <div class="bg-emerald-500 h-1.5 rounded-full" style="width: 75%"></div>
              </div>
              <span class="text-xs font-mono font-medium text-foreground">3 / 4</span>
            </div>
          </td>
          <td class="py-3 px-4 text-right space-x-1">
            <button 
              hx-get="/books/12/edit-modal" 
              hx-target="#modal-slot" 
              class="px-2 py-1 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-muted rounded"
            >
              Edit
            </button>
            <button 
              hx-get="/loans/new?book_id=12" 
              hx-target="#drawer-container" 
              class="px-2 py-1 text-xs font-medium text-primary hover:bg-primary/10 rounded"
            >
              Loan
            </button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</div>
```

---

## 6. Actionable UI/UX Critique Criteria for Component Architects

Every PR or component authored for this application must pass the following 5-point audit:

### Checklist for Topcoat + HTMX Components

1. **Target Specificity & Encapsulation**:
   - [ ] Does the component specify an explicit, minimal `hx-target` (e.g. `hx-target="#row-12"` or `closest tr`) rather than replacing whole pages?
   - [ ] Is an indicator configured via `hx-indicator` or `.htmx-request` CSS styling to prevent perception of freezing during latency?

2. **Keyboard Ergonomics**:
   - [ ] Are primary inputs (`search`, modal dialogs) configured with autofocus?
   - [ ] Can modals and drawers be dismissed with the `Escape` key (`hx-on:keydown.escape="this.remove()"`)?
   - [ ] Are interactive table actions accessible via tab indexing?

3. **Defensive Layout Sizing**:
   - [ ] Do text badges and icons include `shrink-0` to prevent distortion in tight table columns?
   - [ ] Are long book titles and member emails protected with `truncate` or `line-clamp-1`?
   - [ ] Are tabular data columns (ISBN, numbers, dates) rendered in `font-mono tabular-nums`?

4. **Multi-Region Cohesion (`hx-swap-oob`)**:
   - [ ] When state alters global numbers (e.g., returning a book reduces overdue count), is an OOB fragment returned for the sidebar badge?
   - [ ] Does the operation fire a non-intrusive toast notification via `#toast-container` OOB swap?

5. **Error & Empty State Resilience**:
   - [ ] Does every search and table view have a graceful empty state when 0 rows match?
   - [ ] Are form validation errors rendered inline under the faulty input with `aria-invalid="true"` and `text-destructive` rather than generic popups?
