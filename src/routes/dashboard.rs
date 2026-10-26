use std::sync::Arc;
use topcoat::{
    context::{app_context, Cx},
    router::page,
    view::{view, View},
    Error, Result,
};

use crate::auth::guard::require_auth;
use crate::cache::AppCache;
use crate::components::badge::status_badge;
use crate::components::icons::{
    icon_alert_triangle, icon_book_open, icon_circulation, icon_clock, icon_members,
};
use crate::db::DbPool;

#[page("/")]
pub async fn dashboard_page(cx: &Cx) -> Result<impl View> {
    require_auth(cx)?;

    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    // Sync overdue loans first to ensure fresh data
    let _ = pool.write(|tx| crate::db::queries::sync_overdue_loans(tx)).await;

    let d_key = cache.versions.dashboard_key();
    let metrics = if let Some(cached) = cache.metrics.get(&d_key) {
        cached
    } else {
        let fresh = pool
            .read(|conn| crate::db::queries::get_dashboard_metrics(conn))
            .await
            .map_err(|e| Error::msg(e.to_string()))?;
        let arc_metrics = Arc::new(fresh);
        cache.metrics.insert(d_key, Arc::clone(&arc_metrics));
        arc_metrics
    };

    let recent_loans = pool
        .read(|conn| crate::db::queries::get_loans_with_details(conn, None, None, 5, 0))
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    Ok(view! { cx =>
        <div class="space-y-6">
            <title hx-swap-oob="true">"Dashboard | Athenaeum Library"</title>
            // Page Header
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-3xl font-bold tracking-tight">"System Overview"</h1>
                        <p class="text-sm text-muted-foreground">"Operational status, circulating catalog metrics, and overdue alerts."</p>
                    </div>
                </div>

                // Metric Cards Grid
                <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    // Total Physical Books
                    <div class="bg-card text-card-foreground border border-border rounded-xl p-6 shadow-sm flex flex-col justify-between">
                        <div class="flex items-center justify-between">
                            <span class="text-sm font-medium text-muted-foreground">"Total Inventory"</span>
                            <div class="p-2 rounded-lg bg-primary/10 text-primary shrink-0">
                                icon_book_open(class: "size-5")
                            </div>
                        </div>
                        <div class="mt-4">
                            <div class="text-3xl font-bold font-mono tabular-nums leading-none">
                                (metrics.total_physical_copies)
                            </div>
                            <p class="text-xs text-muted-foreground mt-1">
                                (metrics.available_physical_copies) " copies currently on shelves (" (metrics.total_titles) " titles)"
                            </p>
                        </div>
                    </div>

                    // Active Loans
                    <div class="bg-card text-card-foreground border border-border rounded-xl p-6 shadow-sm flex flex-col justify-between">
                        <div class="flex items-center justify-between">
                            <span class="text-sm font-medium text-muted-foreground">"Active Checkouts"</span>
                            <div class="p-2 rounded-lg bg-blue-500/10 text-blue-600 dark:text-blue-400 shrink-0">
                                icon_circulation(class: "size-5")
                            </div>
                        </div>
                        <div class="mt-4">
                            <div class="text-3xl font-bold font-mono tabular-nums leading-none">
                                (metrics.active_loans_count)
                            </div>
                            <p class="text-xs text-muted-foreground mt-1">"Items currently in circulation"</p>
                        </div>
                    </div>

                    // Overdue Loans Alert
                    <div class=(if metrics.overdue_loans_count > 0 {
                        "bg-rose-50/50 dark:bg-rose-950/20 text-rose-950 dark:text-rose-100 border border-rose-200 dark:border-rose-900/50 rounded-xl p-6 shadow-sm flex flex-col justify-between"
                    } else {
                        "bg-card text-card-foreground border border-border rounded-xl p-6 shadow-sm flex flex-col justify-between"
                    })>
                        <div class="flex items-center justify-between">
                            <span class="text-sm font-medium text-muted-foreground">"Overdue Loans"</span>
                            <div class=(if metrics.overdue_loans_count > 0 {
                                "p-2 rounded-lg bg-rose-500/20 text-rose-600 dark:text-rose-400 shrink-0"
                            } else {
                                "p-2 rounded-lg bg-muted text-muted-foreground shrink-0"
                            })>
                                icon_alert_triangle(class: "size-5")
                            </div>
                        </div>
                        <div class="mt-4">
                            <div class="text-3xl font-bold font-mono tabular-nums leading-none">
                                (metrics.overdue_loans_count)
                            </div>
                            <p class="text-xs text-rose-600 dark:text-rose-400 mt-1 font-medium">
                                if metrics.overdue_loans_count > 0 {
                                    <span>"Requires librarian follow-up"</span>
                                } else {
                                    <span>"No overdue loans"</span>
                                }
                            </p>
                        </div>
                    </div>

                    // Active Members
                    <div class="bg-card text-card-foreground border border-border rounded-xl p-6 shadow-sm flex flex-col justify-between">
                        <div class="flex items-center justify-between">
                            <span class="text-sm font-medium text-muted-foreground">"Registered Patrons"</span>
                            <div class="p-2 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 shrink-0">
                                icon_members(class: "size-5")
                            </div>
                        </div>
                        <div class="mt-4">
                            <div class="text-3xl font-bold font-mono tabular-nums leading-none">
                                (metrics.total_active_members)
                            </div>
                            <p class="text-xs text-muted-foreground mt-1">"Active patron memberships in good standing"</p>
                        </div>
                    </div>
                </div>

                // Recent Circulation Activity
                <div class="bg-card text-card-foreground border border-border rounded-xl shadow-sm overflow-hidden">
                    <div class="p-6 border-b border-border flex items-center justify-between">
                        <div>
                            <h2 class="text-lg font-semibold tracking-tight">"Active Circulation Queue"</h2>
                            <p class="text-sm text-muted-foreground">"Books currently on loan to patrons"</p>
                        </div>
                        <a
                            href="/loans"
                            class="text-sm font-medium text-primary hover:underline flex items-center gap-1"
                        >
                            <span>"View circulation desk"</span>
                            <span>"&rarr;"</span>
                        </a>
                    </div>

                    <div class="overflow-x-auto">
                        <table class="w-full text-sm text-left">
                            <thead class="text-xs uppercase bg-muted/50 text-muted-foreground border-b border-border">
                                <tr>
                                    <th class="px-6 py-3 font-semibold">"Book Title"</th>
                                    <th class="px-6 py-3 font-semibold">"Patron / Member"</th>
                                    <th class="px-6 py-3 font-semibold">"Loan Date"</th>
                                    <th class="px-6 py-3 font-semibold">"Due Date"</th>
                                    <th class="px-6 py-3 font-semibold">"Status"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-border">
                                if recent_loans.is_empty() {
                                    <tr>
                                        <td colspan="5" class="px-6 py-8 text-center text-muted-foreground">
                                            "No active loans right now. Circulation desk is clear."
                                        </td>
                                    </tr>
                                } else {
                                    for loan in recent_loans {
                                        let is_overdue = loan.status == crate::db::models::LoanStatus::Overdue;
                                        <tr class=(if is_overdue { "bg-rose-50/40 dark:bg-rose-950/20" } else { "hover:bg-muted/40 transition-colors" })>
                                            <td class="px-6 py-4 font-medium text-foreground">
                                                <div>(&loan.book_title)</div>
                                                <div class="text-xs text-muted-foreground font-mono">(&loan.book_isbn)</div>
                                            </td>
                                            <td class="px-6 py-4">
                                                <div class="font-medium text-foreground">(&loan.member_name)</div>
                                                <div class="text-xs text-muted-foreground font-mono">(&loan.member_number)</div>
                                            </td>
                                            <td class="px-6 py-4 font-mono text-muted-foreground">
                                                (&loan.loan_date)
                                            </td>
                                            <td class="px-6 py-4 font-mono">
                                                if is_overdue {
                                                    <div class="text-rose-600 dark:text-rose-400 font-semibold flex items-center gap-1">
                                                        icon_clock(class: "size-3.5 shrink-0")
                                                        <span>(&loan.due_date)</span>
                                                    </div>
                                                } else {
                                                    <div class="text-foreground">
                                                        (&loan.due_date)
                                                    </div>
                                                }
                                            </td>
                                            <td class="px-6 py-4">
                                                status_badge(variant: loan.status.into())
                                            </td>
                                        </tr>
                                    }
                                }
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
    })
}
