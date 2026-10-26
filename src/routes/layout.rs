use http::{header::HeaderName, HeaderValue};
use topcoat::{
    asset::asset,
    context::Cx,
    cookie::{cookies, Cookies},
    htmx::hx_request,
    router::{
        error::see_other,
        layout,
        response::response_headers,
        Slot,
    },
    view::{view, View},
    Result,
};

use crate::auth::guard::current_user;
use crate::components::icons::{
    icon_book_open, icon_circulation, icon_dashboard, icon_library, icon_log_out, icon_members,
    icon_menu, icon_moon, icon_sun, icon_x,
};

#[layout("/")]
pub async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let path = topcoat::router::request::uri(cx).path();
    let is_login = path == "/login";

    let user = current_user(cx);

    let is_dark = cookies(cx).get("theme").map(|c| c.value() == "dark").unwrap_or(false);
    let html_class = if is_dark { "dark" } else { "" };

    // If attempting to access any non-login route without authentication, redirect to /login
    if !is_login && user.is_none() {
        if hx_request(cx) {
            response_headers(cx).append(
                HeaderName::from_static("hx-redirect"),
                HeaderValue::from_static("/login"),
            );
        }
        return Err(see_other("/login").into());
    }

    // Set anti-stale cache-control headers on authenticated views
    response_headers(cx).append(
        http::header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-cache, must-revalidate"),
    );
    response_headers(cx).append(
        http::header::VARY,
        HeaderValue::from_static("Cookie, HX-Request"),
    );

    let username = user.as_ref().map(|u| u.username.as_str()).unwrap_or("Librarian").to_string();
    let role = user.as_ref().map(|u| u.role.as_str()).unwrap_or("Staff").to_string();

    let nav_item_class = |is_active: bool| {
        if is_active {
            "flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium bg-primary text-primary-foreground shadow-sm transition-colors"
        } else {
            "flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        }
    };

    let is_dash = path == "/";
    let is_books = path.starts_with("/books");
    let is_members = path.starts_with("/members");
    let is_loans = path.starts_with("/loans");

    Ok(view! { cx =>
        if is_login {
            (slot)
        } else {
            <!DOCTYPE html>
            <html lang="en" class=(html_class)>
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>"Athenaeum | Modern Library Management"</title>
                    <link rel="stylesheet" href=(topcoat::tailwind::stylesheet!())>
                    <script src=(asset!("static/htmx.min.js")) defer="defer"></script>
                    <script src=(asset!("static/theme-init.js"))></script>
                    <script src=(asset!("static/app.js")) defer="defer"></script>
                </head>
                <body class="bg-background text-foreground antialiased h-screen overflow-hidden flex flex-col relative">
                    // Toast notification container (persistent across HTMX swaps)
                    <div id="toast-container" class="fixed top-4 right-4 z-50 flex flex-col gap-2 pointer-events-none"></div>

                    // Modal dialog container (persistent across HTMX swaps)
                    <div id="modal-container"></div>

                    // Member slide-over drawer container
                    <div id="drawer-container"></div>

                    // Mobile / Tablet Backdrop Overlay
                    <div id="sidebar-backdrop" data-action="close-sidebar" class="fixed inset-0 bg-background/80 backdrop-blur-xs z-30 hidden lg:hidden"></div>

                    // Mobile / Tablet Top Header Bar
                    <header class="lg:hidden h-14 border-b border-border bg-card flex items-center justify-between px-4 shrink-0 z-20">
                        <div class="flex items-center gap-2.5">
                            <button
                                data-controller="sidebar"
                                type="button"
                                class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                                aria-label="Toggle navigation menu"
                            >
                                icon_menu(class: "size-5 shrink-0")
                            </button>
                            <div class="flex items-center gap-2">
                                <div class="size-7 rounded-md bg-primary text-primary-foreground flex items-center justify-center font-bold text-xs shadow-xs shrink-0">
                                    icon_library(class: "size-4")
                                </div>
                                <span class="font-bold text-sm tracking-tight">"Athenaeum"</span>
                            </div>
                        </div>

                        <div class="flex items-center gap-1">
                            <button
                                data-controller="theme"
                                type="button"
                                class="p-2 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                                title="Toggle theme"
                            >
                                <span class="dark:hidden">
                                    icon_moon(class: "size-4 shrink-0")
                                </span>
                                <span class="hidden dark:inline">
                                    icon_sun(class: "size-4 shrink-0")
                                </span>
                            </button>
                        </div>
                    </header>

                    <div class="flex-1 flex h-full overflow-hidden relative">
                        // Responsive Sidebar: Off-canvas drawer on mobile (<1024px), docked sidebar on desktop (>=1024px)
                        <aside
                            id="app-sidebar"
                            class="fixed inset-y-0 left-0 z-40 w-64 border-r border-border bg-card text-card-foreground p-4 flex flex-col justify-between shrink-0 h-full overflow-y-auto -translate-x-full transition-transform duration-300 lg:static lg:translate-x-0"
                        >
                            <div class="space-y-6">
                                // Logo / Brand + Close button on mobile
                                <div class="flex items-center justify-between px-2">
                                    <div class="flex items-center gap-3">
                                        <div class="size-9 rounded-lg bg-primary text-primary-foreground flex items-center justify-center font-bold text-base shadow-sm shrink-0">
                                            icon_library(class: "size-5")
                                        </div>
                                        <div class="flex flex-col">
                                            <span class="font-bold text-base tracking-tight leading-none">"Athenaeum"</span>
                                            <span class="text-xs text-muted-foreground">"Library Manager"</span>
                                        </div>
                                    </div>
                                    <button
                                        data-action="close-sidebar"
                                        type="button"
                                        class="lg:hidden p-1.5 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground cursor-pointer transition-colors"
                                        aria-label="Close sidebar"
                                    >
                                        icon_x(class: "size-4 shrink-0")
                                    </button>
                                </div>

                                // Boosted Navigation
                                <nav
                                    hx-boost="true"
                                    hx-push-url="true"
                                    class="space-y-1"
                                >
                                    <a
                                        href="/"
                                        class=(nav_item_class(is_dash))
                                    >
                                        icon_dashboard(class: "size-4 shrink-0")
                                        <span>"Dashboard"</span>
                                    </a>
                                    <a
                                        href="/books"
                                        class=(nav_item_class(is_books))
                                    >
                                        icon_book_open(class: "size-4 shrink-0")
                                        <span>"Books Catalog"</span>
                                    </a>
                                    <a
                                        href="/members"
                                        class=(nav_item_class(is_members))
                                    >
                                        icon_members(class: "size-4 shrink-0")
                                        <span>"Patrons & Members"</span>
                                    </a>
                                    <a
                                        href="/loans"
                                        class=(nav_item_class(is_loans))
                                    >
                                        icon_circulation(class: "size-4 shrink-0")
                                        <span class="flex-1">"Circulation Desk"</span>
                                    </a>
                                </nav>
                            </div>

                            // Footer / User Profile & Logout (always pinned to viewport bottom)
                            <div class="pt-4 border-t border-border space-y-3 shrink-0">
                                <div class="flex items-center justify-between px-2">
                                    <div class="flex flex-col min-w-0">
                                        <span class="text-sm font-semibold truncate leading-none mb-1">(&username)</span>
                                        <span class="text-xs text-muted-foreground capitalize leading-none">(&role)</span>
                                    </div>
                                    <button
                                        data-controller="theme"
                                        type="button"
                                        class="p-2 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                                        title="Toggle theme"
                                    >
                                        <span class="dark:hidden">
                                            icon_moon(class: "size-4 shrink-0")
                                        </span>
                                        <span class="hidden dark:inline">
                                            icon_sun(class: "size-4 shrink-0")
                                        </span>
                                    </button>
                                </div>

                                <form hx-post="/logout" class="w-full">
                                    <button
                                        type="submit"
                                        class="flex items-center gap-2 w-full px-3 py-2 rounded-lg text-xs font-medium text-destructive hover:bg-destructive/10 transition-colors cursor-pointer"
                                    >
                                        icon_log_out(class: "size-4 shrink-0")
                                        <span>"Sign Out"</span>
                                    </button>
                                </form>
                            </div>
                        </aside>

                        // Main Content Canvas: independently scrollable view area
                        <main id="main-content" class="flex-1 overflow-y-auto bg-muted/20 p-4 sm:p-6 lg:p-8 h-full">
                            <div class="max-w-7xl mx-auto space-y-6">
                                (slot)
                            </div>
                        </main>
                    </div>
                </body>
            </html>
        }
    })
}
