//! Deduplicated Vector Iconography System for Topcoat
//!
//! All icons render accessible, resolution-independent SVGs on a 24x24 coordinate grid
//! with defensive `shrink-0` layout protection.

#![allow(dead_code, non_camel_case_types)]

use topcoat::{
    Result,
    view::{Attributes, View, class, component, view},
};

macro_rules! define_icon {
    ($name:ident, { $($paths:tt)* }) => {
        #[component]
        pub async fn $name(
            #[default("size-4")] class: &'static str,
            #[default] #[into] label: String,
            #[default] mut attrs: Attributes,
        ) -> Result<impl View> {
            let has_label = !label.is_empty();
            let custom_class = attrs.remove("class");
            Ok(view! {
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class=(class!("shrink-0", class, custom_class))
                    aria-hidden=(has_label.then_some("false").or(Some("true")).filter(|_| !has_label))
                    role=(has_label.then_some("img"))
                    aria-label=(has_label.then_some(label.as_str()))
                    (attrs)
                >
                    $($paths)*
                </svg>
            })
        }
    };
}

define_icon!(icon_library, {
    <path d="m16 6 4 14" />
    <path d="M12 6v14" />
    <path d="M8 8v12" />
    <path d="M4 4v16" />
    <path d="M2 20h20" />
});

define_icon!(icon_book_open, {
    <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
    <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
});

define_icon!(icon_dashboard, {
    <rect width="7" height="9" x="3" y="3" rx="1" />
    <rect width="7" height="5" x="14" y="3" rx="1" />
    <rect width="7" height="9" x="14" y="12" rx="1" />
    <rect width="7" height="5" x="3" y="16" rx="1" />
});

define_icon!(icon_members, {
    <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
    <circle cx="9" cy="7" r="4" />
    <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
    <path d="M16 3.13a4 4 0 0 1 0 7.75" />
});

define_icon!(icon_circulation, {
    <path d="m16 3 4 4-4 4" />
    <path d="M20 7H4" />
    <path d="m8 21-4-4 4-4" />
    <path d="M4 17h16" />
});

define_icon!(icon_clock, {
    <circle cx="12" cy="12" r="10" />
    <polyline points="12 6 12 12 16 14" />
});

define_icon!(icon_search, {
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.3-4.3" />
});

define_icon!(icon_plus, {
    <path d="M5 12h14" />
    <path d="M12 5v14" />
});

define_icon!(icon_pencil, {
    <path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
    <path d="m15 5 4 4" />
});

define_icon!(icon_trash, {
    <path d="M3 6h18" />
    <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
    <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
    <line x1="10" x2="10" y1="11" y2="17" />
    <line x1="14" x2="14" y1="11" y2="17" />
});

define_icon!(icon_x, {
    <path d="M18 6 6 18" />
    <path d="m6 6 12 12" />
});

define_icon!(icon_check, {
    <polyline points="20 6 9 17 4 12" />
});

define_icon!(icon_check_circle, {
    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
    <polyline points="22 4 12 14.01 9 11.01" />
});

define_icon!(icon_chevron_down, {
    <path d="m6 9 6 6 6-6" />
});

define_icon!(icon_alert_triangle, {
    <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
    <line x1="12" x2="12" y1="9" y2="13" />
    <line x1="12" x2="12.01" y1="17" y2="17" />
});

define_icon!(icon_sun, {
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2" />
    <path d="M12 20v2" />
    <path d="m4.93 4.93 1.41 1.41" />
    <path d="m17.66 17.66 1.41 1.41" />
    <path d="M2 12h2" />
    <path d="M20 12h2" />
    <path d="m6.34 17.66-1.41 1.41" />
    <path d="m19.07 4.93-1.41 1.41" />
});

define_icon!(icon_moon, {
    <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
});

define_icon!(icon_user, {
    <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
    <circle cx="12" cy="7" r="4" />
});

define_icon!(icon_lock, {
    <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
    <path d="M7 11V7a5 5 0 0 1 10 0v4" />
});

define_icon!(icon_menu, {
    <line x1="4" x2="20" y1="12" y2="12" />
    <line x1="4" x2="20" y1="6" y2="6" />
    <line x1="4" x2="20" y1="18" y2="18" />
});

define_icon!(icon_log_out, {
    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
    <polyline points="16 17 21 12 16 7" />
    <line x1="21" x2="9" y1="12" y2="12" />
});
