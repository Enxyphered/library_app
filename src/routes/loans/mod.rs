pub mod views;

use serde::Deserialize;
use topcoat::{
    context::{app_context, Cx},
    router::{content::Form, page, response::response_headers, route},
    view::{view, View, ViewExt, ViewHandle},
    Error, Result,
};

use crate::auth::guard::require_auth;
use crate::cache::AppCache;
use crate::components::icons::icon_alert_triangle;
use crate::components::select::SelectOption;
use crate::components::toast::{toast_notification, ToastVariant};
use crate::db::{
    models::{LoanStatus, MemberStatus},
    DbPool,
};
use views::{loan_row_item, loans_page_view, loans_rows, modal_issue_loan};

#[derive(Debug, Deserialize, Default)]
pub struct LoansQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReturnLoanForm {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct IssueLoanForm {
    pub book_id: i64,
    pub member_id: i64,
    pub duration_days: i32,
}

#[page("/loans")]
pub async fn loans_page(cx: &Cx) -> Result<impl View> {
    require_auth(cx)?;

    let pool: &DbPool = app_context(cx);

    // Sync overdue loans first
    let _ = pool.write(|tx| crate::db::queries::sync_overdue_loans(tx)).await;

    let loans = pool
        .read(|conn| crate::db::queries::get_loans_with_details(conn, None, None, 100, 0))
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    loans_page_view(cx, loans).await
}

#[route(GET "/loans/table")]
pub async fn loans_table(cx: &Cx, Form(query): Form<LoansQuery>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);

    let loans = pool
        .read(move |conn| {
            let filter = match query.status.as_deref() {
                Some("active") => Some(LoanStatus::Active),
                Some("overdue") => Some(LoanStatus::Overdue),
                Some("returned") => Some(LoanStatus::Returned),
                _ => None,
            };
            crate::db::queries::get_loans_with_details(conn, filter, None, 100, 0)
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    view! { cx =>
        loans_rows(loans: loans)
    }
    .single()
    .await
}

#[route(POST "/loans/return")]
pub async fn loan_return_action(cx: &Cx, Form(form): Form<ReturnLoanForm>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    let user = crate::auth::guard::current_user(cx);
    let librarian_id = user.map(|u| u.id);

    let res = pool
        .write(move |tx| crate::db::circulation::return_book_tx(tx, form.id, librarian_id))
        .await;

    match res {
        Ok(_) => {
            cache.versions.bump_circulation();
            response_headers(cx).append(
                http::header::HeaderName::from_static("hx-trigger"),
                http::HeaderValue::from_static(r#"{"library:data-changed": {"entity": "loans"}}"#),
            );

            let updated_loan = pool
                .read(move |conn| {
                    let list = crate::db::queries::get_loans_with_details(conn, None, None, 1, 0)?;
                    let matching = list.into_iter().find(|l| l.loan_id == form.id);
                    Ok(matching)
                })
                .await
                .map_err(|e| Error::msg(e.to_string()))?;

            if let Some(loan) = updated_loan {
                view! { cx =>
                    loan_row_item(loan: loan)
                    toast_notification(
                        message: "Book returned and inventory updated",
                        variant: ToastVariant::Success,
                    )
                }
                .single()
                .await
            } else {
                view! { cx =>
                    <tr class="bg-muted/40">
                        <td colspan="6" class="px-6 py-4 text-xs text-muted-foreground italic">
                            "Book return recorded successfully"
                        </td>
                    </tr>
                    toast_notification(
                        message: "Book returned successfully",
                        variant: ToastVariant::Success,
                    )
                }
                .single()
                .await
            }
        }
        Err(e) => {
            let err_msg = e.to_string();
            view! { cx =>
                toast_notification(
                    message: "Failed to return book",
                    variant: ToastVariant::Error,
                )
                <div class="text-xs text-destructive p-2">(&err_msg)</div>
            }
            .single()
            .await
        }
    }
}

#[route(GET "/loans/modal/new")]
pub async fn loan_modal_new(cx: &Cx) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);

    let (books, members) = pool
        .read(|conn| {
            let books = crate::db::queries::search_books(conn, None, None, true, 100, 0)?;
            let members = crate::db::queries::search_members(conn, None, None, 100, 0)?;
            Ok((books, members))
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    let initial_book_options = books
        .iter()
        .map(|b| {
            SelectOption::new(
                b.id.to_string(),
                b.title.clone(),
            )
            .with_description(format!("{} available | ISBN: {}", b.available_copies, b.isbn))
        })
        .collect::<Vec<_>>();

    let initial_member_options = members
        .iter()
        .map(|m| {
            let is_active = m.status == MemberStatus::Active;
            SelectOption::new(
                m.id.to_string(),
                m.name.clone(),
            )
            .with_description(format!(
                "{} • {}{}",
                m.membership_number,
                m.email,
                if !is_active { " (SUSPENDED)" } else { "" }
            ))
            .with_disabled(!is_active)
        })
        .collect::<Vec<_>>();

    modal_issue_loan(cx, initial_book_options, initial_member_options).await
}

#[route(POST "/loans/issue")]
pub async fn loan_issue_action(cx: &Cx, Form(form): Form<IssueLoanForm>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    let user = crate::auth::guard::current_user(cx);
    let librarian_id = user.map(|u| u.id);

    let res = pool
        .write(move |tx| {
            crate::db::circulation::issue_loan_tx(
                tx,
                form.book_id,
                form.member_id,
                librarian_id,
                form.duration_days,
            )
        })
        .await;

    match res {
        Ok(_) => {
            cache.versions.bump_circulation();
            response_headers(cx).append(
                http::header::HeaderName::from_static("hx-trigger"),
                http::HeaderValue::from_static(r#"{"library:data-changed": {"entity": "loans"}}"#),
            );
            view! { cx =>
                <div id="modal-container" hx-swap-oob="innerHTML"></div>
                toast_notification(
                    message: "Book loan successfully issued",
                    variant: ToastVariant::Success,
                )
            }
            .single()
            .await
        }
        Err(e) => {
            let err_msg = e.to_string();
            view! { cx =>
                <div class="rounded-md bg-destructive/15 p-3 text-destructive border border-destructive/20 flex items-center gap-2 text-sm">
                    icon_alert_triangle(class: "size-4 shrink-0")
                    <span>"Checkout failed: " (&err_msg)</span>
                </div>
            }
            .single()
            .await
        }
    }
}
