use topcoat::{
    view::{class, component, view, View},
    Result,
};

use crate::db::models::{LoanStatus, MemberStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Active,
    Overdue,
    Returned,
    Suspended,
}

impl BadgeVariant {
    pub fn as_classes(&self) -> &'static str {
        match self {
            BadgeVariant::Active => "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20",
            BadgeVariant::Overdue => "bg-destructive/10 text-destructive border-destructive/20 font-semibold",
            BadgeVariant::Returned => "bg-muted text-muted-foreground border-border",
            BadgeVariant::Suspended => "bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20 font-semibold",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BadgeVariant::Active => "Active",
            BadgeVariant::Overdue => "Overdue",
            BadgeVariant::Returned => "Returned",
            BadgeVariant::Suspended => "Suspended",
        }
    }
}

impl From<LoanStatus> for BadgeVariant {
    fn from(status: LoanStatus) -> Self {
        match status {
            LoanStatus::Active => BadgeVariant::Active,
            LoanStatus::Overdue => BadgeVariant::Overdue,
            LoanStatus::Returned => BadgeVariant::Returned,
        }
    }
}

impl From<MemberStatus> for BadgeVariant {
    fn from(status: MemberStatus) -> Self {
        match status {
            MemberStatus::Active => BadgeVariant::Active,
            MemberStatus::Suspended => BadgeVariant::Suspended,
        }
    }
}

/// Unified status badge for domain entity states.
#[component]
pub async fn status_badge(
    variant: BadgeVariant,
    #[default("")] custom_class: &'static str,
) -> Result<impl View> {
    let classes = variant.as_classes();
    let text = variant.as_str();

    Ok(view! {
        <span class=(class!(
            "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border shrink-0",
            classes,
            custom_class
        ))>
            (text)
        </span>
    })
}
