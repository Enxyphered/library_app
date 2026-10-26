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
use crate::components::select::{select_option_item, SelectOption};
use crate::components::toast::{toast_notification, ToastVariant};
use crate::db::DbPool;
use views::{member_drawer_view, member_row_item, members_page_view, members_rows, modal_register_member};

#[derive(Debug, Deserialize, Default)]
pub struct MembersQuery {
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMemberForm {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub membership_number: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleStatusForm {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct MemberDrawerQuery {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct MemberSelectQuery {
    pub q: Option<String>,
}

#[page("/members")]
pub async fn members_page(cx: &Cx) -> Result<impl View> {
    require_auth(cx)?;

    let pool: &DbPool = app_context(cx);

    let members = pool
        .read(|conn| crate::db::queries::search_members(conn, None, None, 100, 0))
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    members_page_view(cx, members).await
}

#[route(GET "/members/table")]
pub async fn members_table(cx: &Cx, Form(query): Form<MembersQuery>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);

    let members = pool
        .read(move |conn| crate::db::queries::search_members(conn, query.search.as_deref(), None, 100, 0))
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    view! { cx =>
        members_rows(members: members)
    }
    .single()
    .await
}

#[route(POST "/members/toggle-status")]
pub async fn member_toggle_status(cx: &Cx, Form(form): Form<ToggleStatusForm>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    let member_id = form.id;
    let new_member = pool
        .write(move |tx| {
            let _ = crate::db::queries::toggle_member_status(tx, member_id)?;
            let member = crate::db::queries::get_member_by_id(tx, member_id)?
                .ok_or_else(|| crate::db::DbError::NotFound { entity: "Member", id: member_id.to_string() })?;
            Ok(member)
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    cache.versions.bump_members();
    response_headers(cx).append(
        http::header::HeaderName::from_static("hx-trigger"),
        http::HeaderValue::from_static(r#"{"library:data-changed": {"entity": "members"}}"#),
    );

    view! { cx =>
        member_row_item(member: new_member)
        toast_notification(
            message: "Patron status updated successfully",
            variant: ToastVariant::Success,
        )
    }
    .single()
    .await
}

#[route(GET "/members/modal/new")]
pub async fn member_modal_new(cx: &Cx) -> Result<ViewHandle> {
    require_auth(cx)?;
    modal_register_member(cx).await
}

#[route(POST "/members/create")]
pub async fn member_create_action(cx: &Cx, Form(form): Form<CreateMemberForm>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    let res = pool
        .write(move |tx| {
            let phone_opt = form.phone.as_deref().filter(|p| !p.trim().is_empty());
            crate::db::queries::create_member(
                tx,
                form.name.trim(),
                form.email.trim(),
                phone_opt,
                form.membership_number.trim(),
            )
        })
        .await;

    match res {
        Ok(_) => {
            cache.versions.bump_members();
            response_headers(cx).append(
                http::header::HeaderName::from_static("hx-trigger"),
                http::HeaderValue::from_static(r#"{"library:data-changed": {"entity": "members"}}"#),
            );
            view! { cx =>
                <div id="modal-container" hx-swap-oob="innerHTML"></div>
                toast_notification(
                    message: "Patron successfully registered",
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
                    <span>"Failed to register patron: " (&err_msg)</span>
                </div>
            }
            .single()
            .await
        }
    }
}

#[route(GET "/members/drawer")]
pub async fn member_drawer(cx: &Cx, Form(query): Form<MemberDrawerQuery>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);

    let (member, loans) = pool
        .read(move |conn| {
            let member = crate::db::queries::get_member_by_id(conn, query.id)?
                .ok_or_else(|| crate::db::DbError::NotFound { entity: "Member", id: query.id.to_string() })?;
            let loans = crate::db::queries::get_loans_with_details(conn, None, Some(query.id), 50, 0)?;
            Ok((member, loans))
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    member_drawer_view(cx, member, loans).await
}

#[route(GET "/members/select-options")]
pub async fn members_select_options(cx: &Cx, Form(query): Form<MemberSelectQuery>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);

    let q_trimmed = query.q.filter(|s| !s.trim().is_empty());

    let members = pool
        .read(move |conn| {
            crate::db::queries::search_members(
                conn,
                q_trimmed.as_deref(),
                Some(crate::db::models::MemberStatus::Active),
                20,
                0,
            )
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    view! { cx =>
        if members.is_empty() {
            <div class="px-3 py-2 text-xs text-muted-foreground text-center">
                "No active patrons match your search"
            </div>
        } else {
            for m in members {
                let opt = SelectOption::new(
                    m.id.to_string(),
                    format!("{} ({})", m.name, m.membership_number),
                ).with_description(m.email.clone());

                select_option_item(option: opt)
            }
        }
    }
    .single()
    .await
}
