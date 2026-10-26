use topcoat::{
    context::Cx,
    view::{component, view, View, ViewHandle, ViewExt},
    Result,
};

use crate::components::badge::status_badge;
use crate::components::icons::{icon_check_circle, icon_clock, icon_plus};
use crate::components::modal::modal_dialog;
use crate::components::select::{custom_select, SelectOption};
use crate::db::models::{LoanStatus, LoanWithDetails};

pub async fn loans_page_view(cx: &Cx, loans: Vec<LoanWithDetails>) -> Result<impl View> {
    Ok(view! { cx =>
        <div class="space-y-6">
            <title hx-swap-oob="true">"Circulation Desk | Athenaeum Library"</title>
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight">"Circulation Desk"</h1>
                    <p class="text-sm text-muted-foreground">"Process book loans, manage active checkouts, and process returns."</p>
                </div>
                <button
                    hx-get="/loans/modal/new"
                    hx-target="#modal-container"
                    hx-swap="innerHTML"
                    class="inline-flex items-center gap-2 rounded-lg text-sm font-semibold transition-colors bg-primary text-primary-foreground shadow hover:bg-primary/90 h-10 px-4 py-2 cursor-pointer"
                >
                    icon_plus(class: "size-4 shrink-0")
                    <span>"Issue New Loan"</span>
                </button>
            </div>

            // Filter Tabs with URL history preservation
            <div class="flex items-center gap-2 border-b border-border pb-2">
                <button
                    hx-get="/loans/table"
                    hx-target="#loans-table-body"
                    hx-push-url="/loans"
                    class="px-3 py-1.5 rounded-lg text-sm font-medium hover:bg-muted transition-colors cursor-pointer"
                >
                    "All Circulation"
                </button>
                <button
                    hx-get="/loans/table?status=active"
                    hx-target="#loans-table-body"
                    hx-push-url="/loans?status=active"
                    class="px-3 py-1.5 rounded-lg text-sm font-medium hover:bg-muted transition-colors cursor-pointer"
                >
                    "Active Only"
                </button>
                <button
                    hx-get="/loans/table?status=overdue"
                    hx-target="#loans-table-body"
                    hx-push-url="/loans?status=overdue"
                    class="px-3 py-1.5 rounded-lg text-sm font-medium text-destructive hover:bg-destructive/10 transition-colors cursor-pointer"
                >
                    "Overdue Only"
                </button>
                <button
                    hx-get="/loans/table?status=returned"
                    hx-target="#loans-table-body"
                    hx-push-url="/loans?status=returned"
                    class="px-3 py-1.5 rounded-lg text-sm font-medium text-muted-foreground hover:bg-muted transition-colors cursor-pointer"
                >
                    "Returned Archive"
                </button>
            </div>

            // Loans Table Card
            <div class="bg-card text-card-foreground border border-border rounded-xl shadow-sm overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="w-full text-sm text-left">
                        <thead class="text-xs uppercase bg-muted/50 text-muted-foreground border-b border-border">
                            <tr>
                                <th class="px-6 py-3 font-semibold">"Book Title"</th>
                                <th class="px-6 py-3 font-semibold">"Borrower Patron"</th>
                                <th class="px-6 py-3 font-semibold">"Checkout Date"</th>
                                <th class="px-6 py-3 font-semibold">"Due Date"</th>
                                <th class="px-6 py-3 font-semibold">"Status"</th>
                                <th class="px-6 py-3 font-semibold text-right">"Action"</th>
                            </tr>
                        </thead>
                        <tbody id="loans-table-body" class="divide-y divide-border">
                            loans_rows(loans: loans)
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    })
}

#[component]
pub async fn loans_rows(loans: Vec<LoanWithDetails>) -> Result<impl View> {
    Ok(view! {
        if loans.is_empty() {
            <tr>
                <td colspan="6" class="px-6 py-8 text-center text-muted-foreground">
                    "No loans match this filter criteria."
                </td>
            </tr>
        } else {
            for loan in loans {
                loan_row_item(loan: loan)
            }
        }
    })
}

#[component]
pub async fn loan_row_item(loan: LoanWithDetails) -> Result<impl View> {
    let loan_id = loan.loan_id;
    let is_overdue = loan.status == LoanStatus::Overdue;
    let is_active = loan.status == LoanStatus::Active || is_overdue;

    Ok(view! {
        <tr id=(format!("loan-row-{loan_id}")) class=(if is_overdue { "bg-destructive/5" } else { "hover:bg-muted/40 transition-colors" })>
            <td class="px-6 py-4">
                <div class="font-medium text-foreground">(&loan.book_title)</div>
                <div class="text-xs text-muted-foreground font-mono">(&loan.book_isbn)</div>
            </td>
            <td class="px-6 py-4">
                <div class="font-medium text-foreground">(&loan.member_name)</div>
                <div class="text-xs text-muted-foreground font-mono">(&loan.member_number)</div>
            </td>
            <td class="px-6 py-4 font-mono text-xs text-muted-foreground">
                (&loan.loan_date)
            </td>
            <td class="px-6 py-4 font-mono text-xs">
                if is_overdue {
                    <div class="text-destructive font-semibold flex items-center gap-1">
                        icon_clock(class: "size-3.5 shrink-0")
                        <span>(&loan.due_date)</span>
                        <span class="text-xs font-normal">"(" (loan.days_overdue) "d overdue)"</span>
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
            <td class="px-6 py-4 text-right">
                if is_active {
                    <form
                        hx-post="/loans/return"
                        hx-target=(format!("#loan-row-{loan_id}"))
                        hx-swap="outerHTML"
                        class="inline-block"
                    >
                        <input type="hidden" name="id" value=(loan_id) />
                        <button
                            type="submit"
                            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white shadow-xs cursor-pointer transition-colors"
                        >
                            icon_check_circle(class: "size-3.5 shrink-0")
                            <span>"Check In"</span>
                        </button>
                    </form>
                } else {
                    <span class="text-xs text-muted-foreground italic">
                        "Closed on " (loan.return_date.as_deref().unwrap_or("-"))
                    </span>
                }
            </td>
        </tr>
    })
}

pub async fn modal_issue_loan(
    cx: &Cx,
    initial_book_options: Vec<SelectOption>,
    initial_member_options: Vec<SelectOption>,
) -> Result<ViewHandle> {
    view! { cx =>
        modal_dialog(
            title: "Issue New Book Loan",
            description: Some("Select an available book and an active library member."),
        ) {
            <div id="loan-create-alert"></div>

            <form
                hx-post="/loans/issue"
                hx-target="#loan-create-alert"
                hx-swap="innerHTML"
                class="space-y-4"
            >
                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Book Title / Catalog Copy"</label>
                    custom_select(
                        name: "book_id",
                        placeholder: "Select or search book...",
                        options: initial_book_options,
                        searchable: true,
                        hx_get: "/books/select-options",
                    )
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Borrowing Patron / Member"</label>
                    custom_select(
                        name: "member_id",
                        placeholder: "Select or search patron...",
                        options: initial_member_options,
                        searchable: true,
                        hx_get: "/members/select-options",
                    )
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Loan Duration (Days)"</label>
                    <input
                        type="number"
                        name="duration_days"
                        required="required"
                        min="1"
                        max="60"
                        value="14"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                    <p class="text-xs text-muted-foreground">"Standard library circulation duration is 14 days."</p>
                </div>

                <div class="flex justify-end gap-3 pt-4 border-t border-border">
                    <button
                        type="button"
                        data-action="close-modal"
                        class="px-4 py-2 rounded-md text-sm font-medium border border-input hover:bg-muted cursor-pointer transition-colors"
                    >
                        "Cancel"
                    </button>
                    <button
                        type="submit"
                        class="px-4 py-2 rounded-md text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 shadow cursor-pointer font-semibold"
                    >
                        "Confirm Checkout"
                    </button>
                </div>
            </form>
        }
    }
    .single()
    .await
}
