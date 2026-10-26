use topcoat::{
    view::{class, component, view, View},
    Result,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Success,
    Error,
    #[allow(dead_code)]
    Info,
}

impl ToastVariant {
    pub fn as_classes(&self) -> &'static str {
        match self {
            ToastVariant::Success => "bg-primary text-primary-foreground border-primary/20",
            ToastVariant::Error => "bg-destructive text-destructive-foreground border-destructive/20",
            ToastVariant::Info => "bg-card text-card-foreground border-border",
        }
    }
}

/// Out-of-band toast notification rendered directly into `#toast-container`.
#[component]
pub async fn toast_notification(
    #[default("Action completed successfully")] message: &'static str,
    #[default(ToastVariant::Success)] variant: ToastVariant,
) -> Result<impl View> {
    let variant_classes = variant.as_classes();
    Ok(view! {
        <div id="toast-container" hx-swap-oob="beforeend">
            <div
                data-controller="toast"
                data-duration="680"
                class=(class!(
                    "pointer-events-auto flex items-center justify-between gap-3 px-4 py-3 rounded-lg border shadow-lg text-sm font-medium animate-in fade-in slide-in-from-top-2 duration-200",
                    variant_classes
                ))
            >
                <span>(message)</span>
                <button
                    type="button"
                    data-action="dismiss"
                    class="p-1 rounded-md opacity-70 hover:opacity-100 transition-opacity cursor-pointer"
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
        </div>
    })
}
