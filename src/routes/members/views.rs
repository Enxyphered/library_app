use topcoat::{
    context::Cx,
    view::{class, component, view, View, ViewHandle, ViewExt},
    Result,
};

use crate::components::badge::status_badge;
use crate::components::icons::{icon_plus, icon_search, icon_x};
use crate::components::modal::modal_dialog;
use crate::db::models::{LoanWithDetails, Member, MemberStatus};

pub async fn members_page_view(cx: &Cx, members: Vec<Member>) -> Result<impl View> {
    Ok(view! { cx =>
        <div class="space-y-6">
            <title hx-swap-oob="true">"Patrons & Members | Athenaeum Library"</title>
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight">"Patrons & Members"</h1>
                    <p class="text-sm text-muted-foreground">"Manage borrower accounts, cards, contact details, and standing."</p>
                </div>
                <button
                    hx-get="/members/modal/new"
                    hx-target="#modal-container"
                    hx-swap="innerHTML"
                    class="inline-flex items-center gap-2 rounded-lg text-sm font-semibold transition-colors bg-primary text-primary-foreground shadow hover:bg-primary/90 h-10 px-4 py-2 cursor-pointer"
                >
                    icon_plus(class: "size-4 shrink-0")
                    <span>"Register New Patron"</span>
                </button>
            </div>

            // Search Toolbar
            <div class="bg-card text-card-foreground border border-border rounded-xl p-4 shadow-sm flex items-center justify-between">
                <div class="relative w-full md:w-96">
                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-muted-foreground">
                        icon_search(class: "size-4 shrink-0")
                    </div>
                    <input
                        type="search"
                        name="search"
                        placeholder="Search by name, email, or card ID..."
                        hx-get="/members/table"
                        hx-trigger="keyup changed delay:300ms, search, library:data-changed from:body"
                        hx-target="#members-table-body"
                        class="flex h-10 w-full rounded-md border border-input bg-background pl-9 pr-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                    />
                </div>
            </div>

            // Members Table Card
            <div class="bg-card text-card-foreground border border-border rounded-xl shadow-sm overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="w-full text-sm text-left">
                        <thead class="text-xs uppercase bg-muted/50 text-muted-foreground border-b border-border">
                            <tr>
                                <th class="px-6 py-3 font-semibold">"Patron Name"</th>
                                <th class="px-6 py-3 font-semibold">"Card / Identifier"</th>
                                <th class="px-6 py-3 font-semibold">"Contact Information"</th>
                                <th class="px-6 py-3 font-semibold">"Standing"</th>
                                <th class="px-6 py-3 font-semibold text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody id="members-table-body" class="divide-y divide-border">
                            members_rows(members: members)
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    })
}

#[component]
pub async fn members_rows(members: Vec<Member>) -> Result<impl View> {
    Ok(view! {
        if members.is_empty() {
            <tr>
                <td colspan="5" class="px-6 py-8 text-center text-muted-foreground">
                    "No patron records found."
                </td>
            </tr>
        } else {
            for member in members {
                member_row_item(member: member)
            }
        }
    })
}

#[component]
pub async fn member_row_item(member: Member) -> Result<impl View> {
    let member_id = member.id;
    let is_active = member.status == MemberStatus::Active;

    Ok(view! {
        <tr id=(format!("member-row-{member_id}")) class="hover:bg-muted/40 transition-colors">
            <td class="px-6 py-4">
                <div class="font-medium text-foreground text-base">(&member.name)</div>
            </td>
            <td class="px-6 py-4 font-mono text-xs text-foreground font-semibold">
                (&member.membership_number)
            </td>
            <td class="px-6 py-4 space-y-0.5">
                <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
                    <span>(&member.email)</span>
                </div>
                if let Some(ref phone) = member.phone {
                    <div class="text-xs text-muted-foreground font-mono">
                        (phone)
                    </div>
                }
            </td>
            <td class="px-6 py-4">
                status_badge(variant: member.status.into())
            </td>
            <td class="px-6 py-4 text-right">
                <div class="flex items-center justify-end gap-2">
                    <button
                        hx-get=(format!("/members/drawer?id={member_id}"))
                        hx-target="#drawer-container"
                        hx-swap="innerHTML"
                        class="h-8 px-3 inline-flex items-center justify-center text-xs font-medium rounded-md border border-input hover:bg-muted transition-colors cursor-pointer shrink-0"
                    >
                        "History & Loans"
                    </button>

                    <form
                        hx-post="/members/toggle-status"
                        hx-target=(format!("#member-row-{member_id}"))
                        hx-swap="outerHTML"
                        class="inline-block"
                    >
                        <input type="hidden" name="id" value=(member_id) />
                        <button
                            type="submit"
                            class=(class!(
                                "h-8 px-3 inline-flex items-center justify-center text-xs font-medium rounded-md transition-colors cursor-pointer shrink-0",
                                if is_active {
                                    "border border-amber-500/30 text-amber-600 hover:bg-amber-500/10 dark:text-amber-400"
                                } else {
                                    "border border-emerald-500/30 text-emerald-600 hover:bg-emerald-500/10 dark:text-emerald-400"
                                }
                            ))
                        >
                            if is_active { "Suspend" } else { "Reactivate" }
                        </button>
                    </form>
                </div>
            </td>
        </tr>
    })
}

pub async fn modal_register_member(cx: &Cx) -> Result<ViewHandle> {
    view! { cx =>
        modal_dialog(
            title: "Register New Patron",
            description: Some("Add a new member to allow issuing books and managing loan records."),
        ) {
            <div id="member-create-alert"></div>

            <form
                hx-post="/members/create"
                hx-target="#member-create-alert"
                hx-swap="innerHTML"
                class="space-y-4"
            >
                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Full Name"</label>
                    <input
                        type="text"
                        name="name"
                        required="required"
                        placeholder="e.g. Eleanor Vance"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Email Address"</label>
                    <input
                        type="email"
                        name="email"
                        required="required"
                        placeholder="e.g. eleanor.vance@example.com"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Phone Number (Optional)"</label>
                    <input
                        type="tel"
                        name="phone"
                        placeholder="e.g. +1-555-0149"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Membership Number / Card Barcode"</label>
                    <input
                        type="text"
                        name="membership_number"
                        required="required"
                        placeholder="e.g. LIB-2026-1045"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono uppercase focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
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
                        "Create Patron Account"
                    </button>
                </div>
            </form>
        }
    }
    .single()
    .await
}

pub async fn member_drawer_view(cx: &Cx, member: Member, loans: Vec<LoanWithDetails>) -> Result<ViewHandle> {
    view! { cx =>
        <div id="drawer-backdrop" class="fixed inset-0 z-50 flex justify-end bg-black/50 backdrop-blur-xs animate-in fade-in duration-200">
            <div class="bg-card text-card-foreground border-l border-border w-full max-w-md h-full flex flex-col shadow-2xl animate-in slide-in-from-right duration-200">
                // Drawer Header
                <div class="p-6 border-b border-border flex items-center justify-between">
                    <div>
                        <h3 class="text-lg font-bold tracking-tight">(&member.name)</h3>
                        <p class="text-xs font-mono text-muted-foreground">(&member.membership_number)</p>
                    </div>
                    <button
                        type="button"
                        data-action="close-modal"
                        class="p-1 rounded-md text-muted-foreground hover:bg-muted cursor-pointer transition-colors"
                    >
                        icon_x(class: "size-5 shrink-0")
                    </button>
                </div>

                // Drawer Body
                <div class="flex-1 overflow-y-auto p-6 space-y-6">
                    // Patron Info Summary
                    <div class="bg-muted/40 p-4 rounded-lg border border-border space-y-2">
                        <div class="text-xs text-muted-foreground">"Email: " <span class="text-foreground font-medium">(&member.email)</span></div>
                        <div class="text-xs text-muted-foreground">"Status: " 
                            if member.status == MemberStatus::Active {
                                <span class="text-emerald-600 font-semibold">"Active"</span>
                            } else {
                                <span class="text-rose-600 font-semibold">"Suspended"</span>
                            }
                        </div>
                        if let Some(ref phone) = member.phone {
                            <div class="text-xs text-muted-foreground">"Phone: " <span class="text-foreground font-mono">(phone)</span></div>
                        }
                    </div>

                    // Borrowing History & Active Loans
                    <div class="space-y-3">
                        <h4 class="text-sm font-semibold tracking-tight">"Borrowing History & Active Loans"</h4>
                        if loans.is_empty() {
                            <div class="text-xs text-muted-foreground text-center py-6 border border-dashed border-border rounded-lg">
                                "No active or past loans found for this patron."
                            </div>
                        } else {
                            <div class="space-y-2.5">
                                for loan in loans {
                                    <div class="p-3 rounded-lg border border-border bg-card space-y-1.5 shadow-xs">
                                        <div class="flex items-start justify-between gap-2">
                                            <div class="font-medium text-sm text-foreground leading-tight">
                                                (&loan.book_title)
                                            </div>
                                            status_badge(variant: loan.status.into())
                                        </div>
                                        <div class="text-xs font-mono text-muted-foreground">
                                            "ISBN: " (&loan.book_isbn)
                                        </div>
                                        <div class="flex items-center justify-between text-xs pt-1 border-t border-border/50 text-muted-foreground">
                                            <span>"Borrowed: " (&loan.loan_date)</span>
                                            <span>"Due: " (&loan.due_date)</span>
                                        </div>
                                    </div>
                                }
                            </div>
                        }
                    </div>
                </div>

                // Drawer Footer
                <div class="p-4 border-t border-border flex justify-end">
                    <button
                        type="button"
                        data-action="close-modal"
                        class="px-4 py-2 rounded-md text-sm font-medium border border-input hover:bg-muted cursor-pointer transition-colors"
                    >
                        "Close Drawer"
                    </button>
                </div>
            </div>
        </div>
    }
    .single()
    .await
}
