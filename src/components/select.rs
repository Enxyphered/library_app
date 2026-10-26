use topcoat::{
    Result,
    view::{class, component, view, View},
};

use crate::components::icons::{icon_check, icon_chevron_down, icon_search};

#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub disabled: bool,
    pub selected: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            disabled: false,
            selected: false,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// Renders an individual option item for select / combobox
#[component]
pub async fn select_option_item(
    option: SelectOption,
    #[default] id: Option<String>,
) -> Result<impl View> {
    let opt_id = id.unwrap_or_else(|| format!("opt-{}", option.value));
    let is_selected = option.selected;
    let is_disabled = option.disabled;

    Ok(view! {
        <div
            id=(opt_id)
            role="option"
            tabindex=(if is_disabled { "-1" } else { "0" })
            aria-selected=(if is_selected { "true" } else { "false" })
            data-disabled=(if is_disabled { "true" } else { "false" })
            data-value=(&option.value)
            data-label=(&option.label)
            class=(class!(
                "relative flex w-full select-none items-center rounded-sm py-2 pl-8 pr-3 text-sm outline-none transition-colors",
                "cursor-pointer hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground" if !is_disabled,
                "opacity-50 cursor-not-allowed bg-muted/40" if is_disabled,
                "bg-accent/40 font-medium text-foreground" if is_selected
            ))
        >
            // Checkmark indicator (opacity toggled defensively to prevent layout shift)
            <span
                data-select-check="true"
                class=(class!(
                    "absolute left-2 flex size-4 shrink-0 min-w-4 min-h-4 items-center justify-center text-primary transition-opacity",
                    "opacity-100" if is_selected,
                    "opacity-0" if !is_selected
                ))
            >
                icon_check(class: "size-3.5 shrink-0")
            </span>

            <div class="flex flex-col min-w-0 flex-1">
                <span class="truncate font-medium text-foreground">
                    (&option.label)
                </span>
                if let Some(desc) = option.description {
                    <span class="truncate text-xs text-muted-foreground mt-0.5">
                        (desc)
                    </span>
                }
            </div>
        </div>
    })
}

/// Renders a full custom select / combobox component with Shadcn styling.
#[component]
pub async fn custom_select(
    name: &'static str,
    #[default] id: Option<&'static str>,
    #[default("Select an option...")] placeholder: &'static str,
    #[default] #[into] selected_value: Option<String>,
    #[default] #[into] selected_label: Option<String>,
    #[default] options: Vec<SelectOption>,
    /// Search endpoint (e.g. "/books/select-options") for remote HTMX dynamic filtering
    #[default] #[into] search_url: Option<&'static str>,
    /// Whether to include a search filter input in the popover
    #[default(false)] searchable: bool,
    /// Optional HTMX target for the search input
    #[default] #[into] search_target: Option<String>,
    /// Extra Tailwind classes for the outer wrapper
    #[default] class: &'static str,
    /// Optional HTMX attributes on the hidden input to trigger external swaps when value changes
    #[default] #[into] hx_get: Option<&'static str>,
    #[default] #[into] hx_target: Option<&'static str>,
    #[default] #[into] hx_include: Option<&'static str>,
) -> Result<impl View> {
    let select_id = id.unwrap_or(name);
    let listbox_id = format!("{}-listbox", select_id);
    let target_id = search_target.unwrap_or_else(|| format!("#{}-options-list", select_id));
    let options_list_id = format!("{}-options-list", select_id);

    let current_val = selected_value.unwrap_or_default();
    let has_val = !current_val.is_empty();
    let display_text = selected_label.unwrap_or_else(|| {
        if has_val {
            options
                .iter()
                .find(|o| o.value == current_val)
                .map(|o| o.label.clone())
                .unwrap_or_else(|| placeholder.to_string())
        } else {
            placeholder.to_string()
        }
    });

    Ok(view! {
        <div
            data-controller="custom-select"
            class=(class!("relative w-full", class))
        >
            // Real hidden input to preserve standard HTML form serialization and HTMX trigger events
            <input
                type="hidden"
                id=(select_id)
                name=(name)
                value=(&current_val)
                hx-get=(hx_get)
                hx-trigger=(hx_get.is_some().then_some("change"))
                hx-target=(hx_target)
                hx-include=(hx_include)
            />

            // Trigger Button
            <button
                type="button"
                data-select-trigger="true"
                role="combobox"
                aria-expanded="false"
                aria-haspopup="listbox"
                aria-controls=(&listbox_id)
                class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 text-foreground cursor-pointer transition-colors shadow-sm"
            >
                <span
                    data-select-label="true"
                    class=(class!(
                        "truncate text-left font-normal",
                        "text-foreground" if has_val,
                        "text-muted-foreground" if !has_val
                    ))
                >
                    (display_text)
                </span>
                <span
                    data-select-chevron="true"
                    class="opacity-50 shrink-0 min-w-4 min-h-4 transition-transform duration-200"
                >
                    icon_chevron_down(class: "size-4 shrink-0")
                </span>
            </button>

            // Popover Dropdown Panel
            <div
                data-select-popover="true"
                class="hidden absolute left-0 right-0 top-full mt-1.5 z-50 min-w-[8rem] w-full overflow-hidden rounded-md border border-border bg-popover text-popover-foreground shadow-lg animate-in fade-in-80 zoom-in-95"
            >
                // Optional Search Input (local or HTMX remote)
                if searchable || search_url.is_some() {
                    <div class="flex items-center border-b border-border px-3 py-1.5 bg-popover">
                        icon_search(class: "mr-2 size-4 shrink-0 min-w-4 min-h-4 text-muted-foreground opacity-50")
                        <input
                            type="search"
                            name=(search_url.is_some().then_some("q"))
                            data-select-search="true"
                            autocomplete="off"
                            placeholder="Type to filter..."
                            hx-get=(search_url)
                            hx-trigger=(search_url.is_some().then_some("input changed delay:200ms, search"))
                            hx-target=(search_url.is_some().then_some(target_id.as_str()))
                            hx-swap=(search_url.is_some().then_some("innerHTML"))
                            hx-params=(search_url.is_some().then_some("q"))
                            hx-include=(search_url.is_some().then_some("this"))
                            hx-sync=(search_url.is_some().then_some("this:replace"))
                            class="flex h-8 w-full rounded-md bg-transparent text-sm outline-none placeholder:text-muted-foreground text-foreground border-0 focus:ring-0 focus:outline-none"
                        />
                    </div>
                }

                // Options List Container
                <div
                    id=(&listbox_id)
                    data-select-listbox="true"
                    role="listbox"
                    tabindex="-1"
                    class="max-h-60 overflow-y-auto p-1 divide-y-0 space-y-0.5 overscroll-contain"
                >
                    <div id=(&options_list_id) class="space-y-0.5">
                        if options.is_empty() && search_url.is_none() {
                            <div data-select-empty="true" class="py-6 text-center text-sm text-muted-foreground select-none">
                                "No options available."
                            </div>
                        } else {
                            <div data-select-empty="true" class="hidden py-6 text-center text-sm text-muted-foreground select-none">
                                "No matches found."
                            </div>
                            for opt in options {
                                select_option_item(option: opt)
                            }
                        }
                    </div>
                </div>
            </div>
        </div>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use topcoat::router::{to_bytes, Body, Router};

    #[topcoat::router::page("/test-remote")]
    async fn test_remote_page(cx: &topcoat::context::Cx) -> topcoat::Result<impl topcoat::view::View> {
        let _ = cx;
        Ok(view! {
            custom_select(
                name: "book_id",
                placeholder: "Search title...",
                search_url: "/books/select-options",
                searchable: true,
            )
        })
    }

    #[topcoat::router::page("/test-local")]
    async fn test_local_page(cx: &topcoat::context::Cx) -> topcoat::Result<impl topcoat::view::View> {
        let _ = cx;
        Ok(view! {
            custom_select(
                name: "category",
                placeholder: "Select category...",
                searchable: true,
            )
        })
    }

    #[tokio::test]
    async fn test_custom_select_remote_and_local_search_attributes() {
        let router = Router::builder()
            .page(test_remote_page)
            .page(test_local_page)
            .build();

        let req_remote = http::Request::builder().uri("/test-remote").body(Body::empty()).unwrap();
        let res_remote = router.handle(req_remote).await;
        let bytes_remote = to_bytes(res_remote.into_body(), usize::MAX).await.unwrap();
        let html_remote = String::from_utf8(bytes_remote.to_vec()).unwrap();

        assert!(html_remote.contains("name=\"q\""), "Expected name=\"q\" in search input, got:\n{}", html_remote);
        assert!(html_remote.contains("hx-include=\"this\""), "Expected hx-include=\"this\" in search input");
        assert!(html_remote.contains("hx-params=\"q\""), "Expected hx-params=\"q\" in search input");
        assert!(html_remote.contains("hx-get=\"/books/select-options\""));

        let req_local = http::Request::builder().uri("/test-local").body(Body::empty()).unwrap();
        let res_local = router.handle(req_local).await;
        let bytes_local = to_bytes(res_local.into_body(), usize::MAX).await.unwrap();
        let html_local = String::from_utf8(bytes_local.to_vec()).unwrap();

        assert!(!html_local.contains("name=\"q\""), "Local search should not have name=\"q\"");
        assert!(!html_local.contains("hx-get="), "Local search should not have hx-get");
    }
}
