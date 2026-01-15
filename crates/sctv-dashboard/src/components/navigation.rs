//! Navigation components for the dashboard.
//!
//! Features asymmetric sidebar design with bold typography
//! and geometric accent elements.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use super::icons::{AlertIcon, Logo, PolicyIcon, ProjectsIcon, SettingsIcon};
use super::CountBadge;

/// Navigation item definition.
#[derive(Clone)]
pub struct NavItem {
    pub path: &'static str,
    pub label: &'static str,
    pub badge_count: Option<u32>,
}

/// Main sidebar navigation component.
#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    let current_path = move || location.pathname.get();

    view! {
        <aside class="sidebar">
            <div class="sidebar__header">
                <a href="/" class="sidebar__logo">
                    <Logo/>
                    <span class="sidebar__title">"SCTV"</span>
                </a>
                <span class="sidebar__subtitle">"Supply Chain Trust"</span>
            </div>

            <nav class="sidebar__nav">
                <NavLinkProjects current_path=current_path.clone()/>
                <NavLinkAlerts current_path=current_path.clone()/>
                <NavLinkPolicies current_path=current_path.clone()/>
                <NavLinkSettings current_path=current_path.clone()/>
            </nav>

            <div class="sidebar__footer">
                <div class="sidebar__version">
                    <span class="sidebar__version-label">"Version"</span>
                    <span class="sidebar__version-number">"0.1.0"</span>
                </div>
                <div class="sidebar__geometric"></div>
            </div>
        </aside>
    }
}

/// Projects navigation link.
#[component]
fn NavLinkProjects(current_path: impl Fn() -> String + Send + Sync + Clone + 'static) -> impl IntoView {
    let is_active = move || {
        let path = current_path();
        path == "/projects" || path == "/"
    };

    view! {
        <a
            href="/projects"
            class=move || if is_active() { "nav-link nav-link--active" } else { "nav-link" }
        >
            <span class="nav-link__icon"><ProjectsIcon/></span>
            <span class="nav-link__label">"Projects"</span>
            <span class="nav-link__indicator"></span>
        </a>
    }
}

/// Alerts navigation link.
#[component]
fn NavLinkAlerts(current_path: impl Fn() -> String + Send + Sync + Clone + 'static) -> impl IntoView {
    let is_active = move || current_path() == "/alerts";

    view! {
        <a
            href="/alerts"
            class=move || if is_active() { "nav-link nav-link--active" } else { "nav-link" }
        >
            <span class="nav-link__icon"><AlertIcon/></span>
            <span class="nav-link__label">"Alerts"</span>
            <CountBadge count=12u32/>
            <span class="nav-link__indicator"></span>
        </a>
    }
}

/// Policies navigation link.
#[component]
fn NavLinkPolicies(current_path: impl Fn() -> String + Send + Sync + Clone + 'static) -> impl IntoView {
    let is_active = move || current_path() == "/policies";

    view! {
        <a
            href="/policies"
            class=move || if is_active() { "nav-link nav-link--active" } else { "nav-link" }
        >
            <span class="nav-link__icon"><PolicyIcon/></span>
            <span class="nav-link__label">"Policies"</span>
            <span class="nav-link__indicator"></span>
        </a>
    }
}

/// Settings navigation link.
#[component]
fn NavLinkSettings(current_path: impl Fn() -> String + Send + Sync + Clone + 'static) -> impl IntoView {
    let is_active = move || current_path() == "/settings";

    view! {
        <a
            href="/settings"
            class=move || if is_active() { "nav-link nav-link--active" } else { "nav-link" }
        >
            <span class="nav-link__icon"><SettingsIcon/></span>
            <span class="nav-link__label">"Settings"</span>
            <span class="nav-link__indicator"></span>
        </a>
    }
}

/// Mobile navigation toggle button.
#[component]
pub fn MobileMenuToggle(
    is_open: ReadSignal<bool>,
    toggle: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <button
            class=move || if is_open.get() { "mobile-toggle mobile-toggle--open" } else { "mobile-toggle" }
            on:click=move |_| toggle()
            aria-label="Toggle navigation"
        >
            <span class="mobile-toggle__line"></span>
            <span class="mobile-toggle__line"></span>
            <span class="mobile-toggle__line"></span>
        </button>
    }
}

/// Breadcrumb navigation.
#[component]
pub fn Breadcrumbs(items: Vec<(String, Option<String>)>) -> impl IntoView {
    let items_len = items.len();

    view! {
        <nav class="breadcrumbs" aria-label="Breadcrumb">
            {items
                .into_iter()
                .enumerate()
                .map(|(i, (label, href))| {
                    let is_last = i == items_len - 1;
                    let label_clone = label.clone();
                    view! {
                        {if i > 0 {
                            Some(view! { <span class="breadcrumbs__separator">"/"</span> })
                        } else {
                            None
                        }}
                        {if let Some(link) = href {
                            view! {
                                <a href=link class="breadcrumbs__link">
                                    {label_clone.clone()}
                                </a>
                            }
                                .into_any()
                        } else {
                            view! {
                                <span
                                    class="breadcrumbs__current"
                                    aria-current=if is_last { Some("page") } else { None }
                                >
                                    {label_clone.clone()}
                                </span>
                            }
                                .into_any()
                        }}
                    }
                })
                .collect_view()}
        </nav>
    }
}

/// Tab navigation component.
#[component]
pub fn TabNav(
    tabs: Vec<(String, String)>,
    active_tab: ReadSignal<String>,
    on_change: impl Fn(String) + Send + Sync + Clone + 'static,
) -> impl IntoView {
    view! {
        <div class="tab-nav" role="tablist">
            {tabs
                .into_iter()
                .map(|(id, label)| {
                    let tab_id = id.clone();
                    let tab_id_aria = id.clone();
                    let tab_id_click = id.clone();
                    let on_change = on_change.clone();
                    view! {
                        <button
                            class=move || {
                                if active_tab.get() == tab_id {
                                    "tab-nav__tab tab-nav__tab--active"
                                } else {
                                    "tab-nav__tab"
                                }
                            }
                            role="tab"
                            aria-selected=move || active_tab.get() == tab_id_aria
                            on:click={
                                let on_change = on_change.clone();
                                move |_| on_change(tab_id_click.clone())
                            }
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
            <span class="tab-nav__indicator"></span>
        </div>
    }
}
