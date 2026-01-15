//! SVG icon components for the dashboard.
//!
//! A collection of minimal, geometric icons that complement
//! the bold asymmetric design language.

use leptos::prelude::*;

/// Icon size variants.
#[derive(Debug, Clone, Copy, Default)]
pub enum IconSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl IconSize {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Small => "icon icon--sm",
            Self::Medium => "icon icon--md",
            Self::Large => "icon icon--lg",
        }
    }
}

/// Projects/folder icon.
#[component]
pub fn ProjectsIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V7z"/>
            <path d="M8 12h8M8 15h5"/>
        </svg>
    }
}

/// Alert/warning icon.
#[component]
pub fn AlertIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 9v4m0 4h.01"/>
            <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
        </svg>
    }
}

/// Shield/policy icon.
#[component]
pub fn PolicyIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            <path d="M9 12l2 2 4-4"/>
        </svg>
    }
}

/// Settings/gear icon.
#[component]
pub fn SettingsIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
        </svg>
    }
}

/// Search/magnifier icon.
#[component]
pub fn SearchIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="11" cy="11" r="8"/>
            <path d="M21 21l-4.35-4.35"/>
        </svg>
    }
}

/// Plus/add icon.
#[component]
pub fn PlusIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 5v14M5 12h14"/>
        </svg>
    }
}

/// Check/success icon.
#[component]
pub fn CheckIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 6L9 17l-5-5"/>
        </svg>
    }
}

/// X/close icon.
#[component]
pub fn CloseIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
    }
}

/// Arrow right icon.
#[component]
pub fn ArrowRightIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7"/>
        </svg>
    }
}

/// External link icon.
#[component]
pub fn ExternalLinkIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"/>
        </svg>
    }
}

/// Package/box icon.
#[component]
pub fn PackageIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M16.5 9.4l-9-5.19M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/>
            <path d="M3.27 6.96L12 12.01l8.73-5.05M12 22.08V12"/>
        </svg>
    }
}

/// Clock/time icon.
#[component]
pub fn ClockIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 6v6l4 2"/>
        </svg>
    }
}

/// Activity/pulse icon.
#[component]
pub fn ActivityIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
        </svg>
    }
}

/// Filter icon.
#[component]
pub fn FilterIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
        </svg>
    }
}

/// Refresh/sync icon.
#[component]
pub fn RefreshIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M23 4v6h-6M1 20v-6h6"/>
            <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
        </svg>
    }
}

/// Menu/hamburger icon.
#[component]
pub fn MenuIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 12h18M3 6h18M3 18h18"/>
        </svg>
    }
}

/// Chevron down icon.
#[component]
pub fn ChevronDownIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 9l6 6 6-6"/>
        </svg>
    }
}

/// More/ellipsis icon.
#[component]
pub fn MoreIcon(#[prop(default = IconSize::Medium)] size: IconSize) -> impl IntoView {
    view! {
        <svg class=size.class() viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="1"/>
            <circle cx="19" cy="12" r="1"/>
            <circle cx="5" cy="12" r="1"/>
        </svg>
    }
}

/// SCTV Logo - geometric shield mark.
#[component]
pub fn Logo() -> impl IntoView {
    view! {
        <svg class="logo" viewBox="0 0 48 48" fill="none">
            <path
                d="M24 4L6 12v12c0 11.1 7.7 21.4 18 24 10.3-2.6 18-12.9 18-24V12L24 4z"
                fill="currentColor"
                opacity="0.1"
            />
            <path
                d="M24 4L6 12v12c0 11.1 7.7 21.4 18 24 10.3-2.6 18-12.9 18-24V12L24 4z"
                stroke="currentColor"
                stroke-width="2"
                fill="none"
            />
            <path d="M16 24l5 5 11-11" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}
