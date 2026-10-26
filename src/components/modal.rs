use topcoat::{
    view::{component, view, Child, View},
    Result,
};

/// Reusable modal dialog wrapper with backdrop dismissal and close button.
#[component]
pub async fn modal_dialog(
    #[default("Dialog")] title: &'static str,
    #[default(None)] description: Option<&'static str>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div id="modal-container" hx-swap-oob="innerHTML">
            <div
                id="modal-backdrop"
                class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-xs animate-in fade-in duration-150"
            >
                <div class="bg-card text-card-foreground border border-border rounded-xl shadow-xl w-full max-w-lg p-6 space-y-6 animate-in zoom-in-95 duration-150">
                    <div class="flex items-center justify-between pb-4 border-b border-border">
                        <div>
                            <h3 class="text-lg font-semibold tracking-tight">(title)</h3>
                            if let Some(desc) = description {
                                <p class="text-sm text-muted-foreground mt-0.5">(desc)</p>
                            }
                        </div>
                        <button
                            type="button"
                            data-action="close-modal"
                            class="p-1.5 rounded-md text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer transition-colors"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class="size-4 shrink-0"
                            >
                                <path d="M18 6 6 18M6 6l12 12" />
                            </svg>
                        </button>
                    </div>

                    (child)
                </div>
            </div>
        </div>
    })
}
