//! Layout components providing the structural foundation.
//!
//! Features bold asymmetric grid layouts with generous
//! white space and geometric accent elements.

use leptos::prelude::*;

use super::navigation::Sidebar;

/// Main layout wrapper with sidebar and content area.
#[component]
pub fn MainLayout(children: Children) -> impl IntoView {
    view! {
        <div class="app-layout">
            <Sidebar/>
            <main class="main-content">
                <div class="main-content__inner">
                    {children()}
                </div>
                <div class="main-content__geometric-accent"></div>
            </main>
        </div>
    }
}

/// Page header with title, subtitle, and actions.
#[component]
pub fn PageHeader(
    #[prop(into)] title: String,
    #[prop(optional)] subtitle: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <header class="page-header">
            <div class="page-header__text">
                <h1 class="page-header__title">{title}</h1>
                {subtitle.map(|s| view! { <p class="page-header__subtitle">{s}</p> })}
            </div>
            {children.map(|c| view! { <div class="page-header__actions">{c()}</div> })}
        </header>
    }
}

/// Section container with optional title.
#[component]
pub fn Section(
    #[prop(optional)] title: Option<String>,
    #[prop(optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let class_name = format!("section {}", class.unwrap_or_default());

    view! {
        <section class=class_name>
            {title.map(|t| view! { <h2 class="section__title">{t}</h2> })}
            <div class="section__content">{children()}</div>
        </section>
    }
}

/// Asymmetric grid layout for dashboard items.
#[component]
pub fn AsymmetricGrid(
    #[prop(optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let class_name = format!("asymmetric-grid {}", class.unwrap_or_default());

    view! {
        <div class=class_name>{children()}</div>
    }
}

/// Grid item with span control.
#[component]
pub fn GridItem(
    #[prop(default = 1)] col_span: u8,
    #[prop(default = 1)] row_span: u8,
    #[prop(optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let style = format!("grid-column: span {col_span}; grid-row: span {row_span};");

    view! {
        <div class=format!("grid-item {}", class.unwrap_or_default()) style=style>
            {children()}
        </div>
    }
}

/// Stats row for overview metrics.
#[component]
pub fn StatsRow(children: Children) -> impl IntoView {
    view! {
        <div class="stats-row">{children()}</div>
    }
}

/// Individual stat display with large typography.
#[component]
pub fn StatCard(
    #[prop(into)] value: String,
    #[prop(into)] label: String,
    #[prop(optional)] trend: Option<(f64, bool)>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    view! {
        <div class=format!("stat-card {}", class.unwrap_or_default())>
            <span class="stat-card__value">{value}</span>
            <span class="stat-card__label">{label}</span>
            {trend.map(|(change, is_positive)| {
                view! {
                    <span class=if is_positive {
                        "stat-card__trend stat-card__trend--up"
                    } else {
                        "stat-card__trend stat-card__trend--down"
                    }>
                        {if is_positive { "+" } else { "" }}
                        {format!("{change:.1}%")}
                    </span>
                }
            })}
        </div>
    }
}

/// Empty state placeholder.
#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(optional)] description: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-state__geometric"></div>
            <h3 class="empty-state__title">{title}</h3>
            {description.map(|d| view! { <p class="empty-state__description">{d}</p> })}
            {children.map(|c| view! { <div class="empty-state__actions">{c()}</div> })}
        </div>
    }
}

/// Loading skeleton placeholder.
#[component]
pub fn Skeleton(
    #[prop(default = "100%")] width: &'static str,
    #[prop(default = "1rem")] height: &'static str,
) -> impl IntoView {
    view! {
        <div class="skeleton" style=format!("width: {}; height: {};", width, height)></div>
    }
}

/// Content container with max width.
#[component]
pub fn Container(#[prop(optional)] class: Option<String>, children: Children) -> impl IntoView {
    view! {
        <div class=format!("container {}", class.unwrap_or_default())>{children()}</div>
    }
}

/// Divider with optional label.
#[component]
pub fn Divider(#[prop(optional)] label: Option<String>) -> impl IntoView {
    view! {
        <div class="divider">
            {label.map(|l| view! { <span class="divider__label">{l}</span> })}
        </div>
    }
}

/// Floating action panel at bottom of viewport.
#[component]
pub fn FloatingPanel(children: Children) -> impl IntoView {
    view! {
        <div class="floating-panel">{children()}</div>
    }
}
