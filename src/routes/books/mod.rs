pub mod views;

use std::sync::Arc;
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
use views::{books_page_view, books_table_rows, modal_edit_book, modal_new_book};

#[derive(Debug, Deserialize, Default)]
pub struct BooksQuery {
    pub search: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookForm {
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub total_copies: i32,
}

#[derive(Debug, Deserialize)]
pub struct EditBookForm {
    pub id: i64,
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub total_copies: i32,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBookForm {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct BookIdQuery {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct SelectSearchQuery {
    pub q: Option<String>,
}

#[page("/books")]
pub async fn books_page(cx: &Cx) -> Result<impl View> {
    require_auth(cx)?;

    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    let cat_key = cache.versions.books_key("categories");
    let categories = if let Some(cached_cats) = cache.categories.get(&cat_key) {
        cached_cats
    } else {
        let fresh_cats = pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare("SELECT DISTINCT category FROM books ORDER BY category ASC")?;
                let cats = stmt
                    .query_map([], |row| row.get::<_, String>(0))?
                    .filter_map(std::result::Result::ok)
                    .collect::<Vec<String>>();
                Ok(cats)
            })
            .await
            .map_err(|e| Error::msg(e.to_string()))?;
        let arc_cats = Arc::new(fresh_cats);
        cache.categories.insert(cat_key, Arc::clone(&arc_cats));
        arc_cats
    };

    let books = pool
        .read(|conn| crate::db::queries::search_books(conn, None, None, false, 100, 0))
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    let mut cat_options = vec![
        SelectOption::new("", "All Categories").with_selected(true),
    ];
    for cat in categories.iter() {
        cat_options.push(SelectOption::new(cat.clone(), cat.clone()));
    }

    books_page_view(cx, cat_options, books).await
}

#[route(GET "/books/table")]
pub async fn books_table(cx: &Cx, Form(query): Form<BooksQuery>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);

    let search_q = query.search.filter(|s| !s.trim().is_empty());
    let category_q = query.category.filter(|c| !c.trim().is_empty());

    let books = pool
        .read(move |conn| {
            crate::db::queries::search_books(
                conn,
                search_q.as_deref(),
                category_q.as_deref(),
                false,
                100,
                0,
            )
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    view! { cx =>
        books_table_rows(books: books)
    }
    .single()
    .await
}

#[route(GET "/books/modal/new")]
pub async fn book_modal_new(cx: &Cx) -> Result<ViewHandle> {
    require_auth(cx)?;
    modal_new_book(cx).await
}

#[route(POST "/books/create")]
pub async fn book_create_action(cx: &Cx, Form(form): Form<CreateBookForm>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    let res = pool
        .write(move |tx| {
            crate::db::queries::create_book(
                tx,
                form.isbn.trim(),
                form.title.trim(),
                form.author.trim(),
                form.category.trim(),
                form.total_copies.max(1),
            )
        })
        .await;

    match res {
        Ok(_) => {
            cache.versions.bump_books();
            response_headers(cx).append(
                http::header::HeaderName::from_static("hx-trigger"),
                http::HeaderValue::from_static(r#"{"library:data-changed": {"entity": "books"}}"#),
            );
            view! { cx =>
                <div id="modal-container" hx-swap-oob="innerHTML"></div>
                toast_notification(
                    message: "Book successfully added to catalog",
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
                    <span>"Failed to create book: " (&err_msg)</span>
                </div>
            }
            .single()
            .await
        }
    }
}

#[route(GET "/books/modal/edit")]
pub async fn book_modal_edit(cx: &Cx, Form(query): Form<BookIdQuery>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);

    let book = pool
        .read(move |conn| {
            crate::db::queries::get_book_by_id(conn, query.id)?
                .ok_or_else(|| crate::db::DbError::NotFound { entity: "book", id: query.id.to_string() })
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    modal_edit_book(cx, book).await
}

#[route(POST "/books/edit")]
pub async fn book_edit_action(cx: &Cx, Form(form): Form<EditBookForm>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    let res = pool
        .write(move |tx| {
            crate::db::queries::update_book(
                tx,
                form.id,
                form.isbn.trim(),
                form.title.trim(),
                form.author.trim(),
                form.category.trim(),
                form.total_copies,
            )
        })
        .await;

    match res {
        Ok(_) => {
            cache.versions.bump_books();
            response_headers(cx).append(
                http::header::HeaderName::from_static("hx-trigger"),
                http::HeaderValue::from_static(r#"{"library:data-changed": {"entity": "books"}}"#),
            );
            view! { cx =>
                <div id="modal-container" hx-swap-oob="innerHTML"></div>
                toast_notification(
                    message: "Book details updated successfully",
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
                    <span>"Failed to save changes: " (&err_msg)</span>
                </div>
            }
            .single()
            .await
        }
    }
}

#[route(POST "/books/delete")]
pub async fn book_delete_action(cx: &Cx, Form(form): Form<DeleteBookForm>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);
    let cache: &AppCache = app_context(cx);

    let res = pool
        .write(move |tx| crate::db::queries::delete_book(tx, form.id))
        .await;

    match res {
        Ok(_) => {
            cache.versions.bump_books();
            response_headers(cx).append(
                http::header::HeaderName::from_static("hx-trigger"),
                http::HeaderValue::from_static(r#"{"library:data-changed": {"entity": "books"}}"#),
            );
            view! { cx =>
                toast_notification(
                    message: "Book successfully deleted from catalog",
                    variant: ToastVariant::Success,
                )
            }
            .single()
            .await
        }
        Err(e) => {
            let err_msg = e.to_string();
            view! { cx =>
                toast_notification(
                    message: "Failed to delete book: active loans exist",
                    variant: ToastVariant::Error,
                )
                <div class="text-xs text-destructive p-2">(&err_msg)</div>
            }
            .single()
            .await
        }
    }
}

#[route(GET "/books/select-options")]
pub async fn books_select_options(cx: &Cx, Form(query): Form<SelectSearchQuery>) -> Result<ViewHandle> {
    require_auth(cx)?;
    let pool: &DbPool = app_context(cx);

    let q_trimmed = query.q.filter(|s| !s.trim().is_empty());

    let books = pool
        .read(move |conn| {
            crate::db::queries::search_books(conn, q_trimmed.as_deref(), None, true, 20, 0)
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))?;

    view! { cx =>
        if books.is_empty() {
            <div class="px-3 py-2 text-xs text-muted-foreground text-center">
                "No available books match your search"
            </div>
        } else {
            for book in books {
                let opt = SelectOption::new(
                    book.id.to_string(),
                    format!("{} ({} avail)", book.title, book.available_copies),
                ).with_description(format!("by {} • ISBN: {}", book.author, book.isbn));

                select_option_item(option: opt)
            }
        }
    }
    .single()
    .await
}
