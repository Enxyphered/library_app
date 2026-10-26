use topcoat::{
    context::Cx,
    view::{component, view, View, ViewHandle, ViewExt},
    Result,
};

use crate::components::icons::{
    icon_pencil, icon_plus, icon_search, icon_trash,
};
use crate::components::modal::modal_dialog;
use crate::components::select::{custom_select, SelectOption};
use crate::db::models::Book;

pub async fn books_page_view(cx: &Cx, cat_options: Vec<SelectOption>, books: Vec<Book>) -> Result<impl View> {
    Ok(view! { cx =>
        <div class="space-y-6">
            <title hx-swap-oob="true">"Books Catalog | Athenaeum Library"</title>
            // Header with title and Action button
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight">"Books Catalog"</h1>
                    <p class="text-sm text-muted-foreground">"Manage titles, ISBNs, shelf stock levels, and catalog classifications."</p>
                </div>
                <button
                    hx-get="/books/modal/new"
                    hx-target="#modal-container"
                    hx-swap="innerHTML"
                    class="inline-flex items-center gap-2 rounded-lg text-sm font-semibold transition-colors bg-primary text-primary-foreground shadow hover:bg-primary/90 h-10 px-4 py-2 cursor-pointer"
                >
                    icon_plus(class: "size-4 shrink-0")
                    <span>"Add New Book"</span>
                </button>
            </div>

            // Search and Category Filter Toolbar
            <div class="bg-card text-card-foreground border border-border rounded-xl p-4 shadow-sm flex flex-col md:flex-row gap-4 items-center justify-between">
                <div class="relative w-full md:w-96">
                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-muted-foreground">
                        icon_search(class: "size-4 shrink-0")
                    </div>
                    <input
                        type="search"
                        name="search"
                        placeholder="Search by title, author, or ISBN..."
                        hx-get="/books/table"
                        hx-trigger="keyup changed delay:300ms, search, library:data-changed from:body"
                        hx-target="#books-table-body"
                        hx-include="[name='category']"
                        class="flex h-10 w-full rounded-md border border-input bg-background pl-9 pr-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                    />
                </div>

                <div class="w-full md:w-64">
                    custom_select(
                        name: "category",
                        placeholder: "All Categories",
                        options: cat_options,
                        searchable: true,
                        hx_get: "/books/table",
                        hx_target: "#books-table-body",
                        hx_include: "[name='search']",
                    )
                </div>
            </div>

            // Books Table Card
            <div class="bg-card text-card-foreground border border-border rounded-xl shadow-sm overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="w-full text-sm text-left">
                        <thead class="text-xs uppercase bg-muted/50 text-muted-foreground border-b border-border">
                            <tr>
                                <th class="px-6 py-3 font-semibold">"Title & Author"</th>
                                <th class="px-6 py-3 font-semibold">"Category"</th>
                                <th class="px-6 py-3 font-semibold">"ISBN"</th>
                                <th class="px-6 py-3 font-semibold">"Availability / Stock"</th>
                                <th class="px-6 py-3 font-semibold text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody id="books-table-body" class="divide-y divide-border">
                            books_table_rows(books: books)
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    })
}

#[component]
pub async fn books_table_rows(books: Vec<Book>) -> Result<impl View> {
    Ok(view! {
        if books.is_empty() {
            <tr>
                <td colspan="5" class="px-6 py-12 text-center text-muted-foreground">
                    <div class="flex flex-col items-center justify-center gap-2">
                        icon_search(class: "size-8 text-muted-foreground/50")
                        <p class="font-medium text-base">"No matching books found"</p>
                        <p class="text-xs">"Try adjusting your search terms or clearing category filters."</p>
                    </div>
                </td>
            </tr>
        } else {
            for book in books {
                let is_low_stock = book.available_copies == 0;
                let book_id = book.id;
                <tr id=(format!("book-row-{}", book_id)) class="hover:bg-muted/40 transition-colors">
                    <td class="px-6 py-4 font-medium text-foreground">
                        <div class="font-semibold text-base">(&book.title)</div>
                        <div class="text-xs text-muted-foreground">"by " (&book.author)</div>
                    </td>
                    <td class="px-6 py-4">
                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-secondary text-secondary-foreground border border-border">
                            (&book.category)
                        </span>
                    </td>
                    <td class="px-6 py-4 font-mono text-xs text-muted-foreground">
                        (&book.isbn)
                    </td>
                    <td class="px-6 py-4">
                        <div class="flex items-center gap-2">
                            if is_low_stock {
                                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold bg-rose-100 text-rose-800 dark:bg-rose-900/40 dark:text-rose-300">
                                    "Out of Stock"
                                </span>
                            } else {
                                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300">
                                    (book.available_copies) " of " (book.total_copies) " Available"
                                </span>
                            }
                        </div>
                    </td>
                    <td class="px-6 py-4 text-right">
                        <div class="flex items-center justify-end gap-2">
                            <button
                                hx-get="/books/modal/edit"
                                hx-vals=(format!(r#"{{"id": {}}}"#, book_id))
                                hx-target="#modal-container"
                                hx-swap="innerHTML"
                                class="p-1.5 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground cursor-pointer transition-colors"
                                title="Edit Book"
                            >
                                icon_pencil(class: "size-4 shrink-0")
                            </button>
                            <button
                                hx-post="/books/delete"
                                hx-vals=(format!(r#"{{"id": {}}}"#, book_id))
                                hx-confirm="Are you sure you want to delete this book? This action cannot be undone."
                                hx-target=(format!("#book-row-{}", book_id))
                                hx-swap="outerHTML"
                                class="p-1.5 rounded-md hover:bg-destructive/10 text-muted-foreground hover:destructive cursor-pointer transition-colors"
                                title="Delete Book"
                            >
                                icon_trash(class: "size-4 shrink-0")
                            </button>
                        </div>
                    </td>
                </tr>
            }
        }
    })
}

pub async fn modal_new_book(cx: &Cx) -> Result<ViewHandle> {
    view! { cx =>
        modal_dialog(
            title: "Add New Title to Catalog",
            description: Some("Enter catalog metadata and physical copy inventory levels."),
        ) {
            <div id="book-create-alert"></div>

            <form
                hx-post="/books/create"
                hx-target="#book-create-alert"
                hx-swap="innerHTML"
                class="space-y-4"
            >
                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Title"</label>
                    <input
                        type="text"
                        name="title"
                        required="required"
                        placeholder="e.g. Structure and Interpretation of Computer Programs"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Author"</label>
                    <input
                        type="text"
                        name="author"
                        required="required"
                        placeholder="e.g. Harold Abelson & Gerald Jay Sussman"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-1.5">
                        <label class="text-sm font-medium">"Category"</label>
                        <input
                            type="text"
                            name="category"
                            required="required"
                            placeholder="e.g. Computer Science"
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        />
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-sm font-medium">"ISBN"</label>
                        <input
                            type="text"
                            name="isbn"
                            required="required"
                            placeholder="e.g. 978-0262510875"
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        />
                    </div>
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Total Physical Copies"</label>
                    <input
                        type="number"
                        name="total_copies"
                        required="required"
                        min="1"
                        value="3"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
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
                        "Create Book"
                    </button>
                </div>
            </form>
        }
    }
    .single()
    .await
}

pub async fn modal_edit_book(cx: &Cx, book: Book) -> Result<ViewHandle> {
    let book_id = book.id;
    let min_allowed = book.total_copies - book.available_copies;

    view! { cx =>
        modal_dialog(
            title: "Edit Book Details",
            description: Some("Update title, category, and inventory levels."),
        ) {
            <div id="book-edit-alert"></div>

            <form
                hx-post="/books/edit"
                hx-target="#book-edit-alert"
                hx-swap="innerHTML"
                class="space-y-4"
            >
                <input type="hidden" name="id" value=(book_id) />

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Title"</label>
                    <input
                        type="text"
                        name="title"
                        value=(book.title)
                        required="required"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Author"</label>
                    <input
                        type="text"
                        name="author"
                        value=(book.author)
                        required="required"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-1.5">
                        <label class="text-sm font-medium">"Category"</label>
                        <input
                            type="text"
                            name="category"
                            value=(book.category)
                            required="required"
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        />
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-sm font-medium">"ISBN"</label>
                        <input
                            type="text"
                            name="isbn"
                            value=(book.isbn)
                            required="required"
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        />
                    </div>
                </div>

                <div class="space-y-1.5">
                    <label class="text-sm font-medium">"Total Copies"</label>
                    <input
                        type="number"
                        name="total_copies"
                        min=(min_allowed)
                        value=(book.total_copies)
                        required="required"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    />
                    <p class="text-xs text-muted-foreground">
                        "Currently " (min_allowed) " copies on loan. Total cannot be less than borrowed count."
                    </p>
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
                        "Save Changes"
                    </button>
                </div>
            </form>
        }
    }
    .single()
    .await
}
