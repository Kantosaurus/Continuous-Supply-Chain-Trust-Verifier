//! Card components for displaying data in bold, memorable layouts.
//!
//! These cards feature asymmetric designs, geometric accents,
//! and high-contrast status indicators.

use leptos::prelude::*;

use super::icons::{ArrowRightIcon, ExternalLinkIcon, MoreIcon, PackageIcon};
use super::{EcosystemBadge, SeverityTag, SlsaLevel, StatusBadge, StatusDot, StatusLevel};

/// Base card component with geometric accent.
#[component]
pub fn Card(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] hoverable: bool,
    children: Children,
) -> impl IntoView {
    let class_name = format!(
        "card {} {}",
        class.unwrap_or_default(),
        if hoverable { "card--hoverable" } else { "" }
    );

    view! {
        <div class=class_name>{children()}</div>
    }
}

/// Project card with status, stats, and ecosystems.
#[component]
pub fn ProjectCard(
    #[prop(into)] name: String,
    #[prop(optional)] description: Option<String>,
    #[prop(into)] status: StatusLevel,
    #[prop(into)] dependency_count: u32,
    #[prop(into)] alert_count: u32,
    #[prop(into)] ecosystems: Vec<String>,
    #[prop(optional)] last_scan: Option<String>,
    #[prop(optional)] slsa_level: Option<u8>,
) -> impl IntoView {
    view! {
        <article class="project-card">
            <div class="project-card__header">
                <div class="project-card__status-corner">
                    <StatusDot level=status/>
                </div>
                <div class="project-card__title-group">
                    <h3 class="project-card__name">{name}</h3>
                    {description.map(|d| view! { <p class="project-card__description">{d}</p> })}
                </div>
                <button class="project-card__menu" aria-label="More options">
                    <MoreIcon/>
                </button>
            </div>

            <div class="project-card__ecosystems">
                {ecosystems
                    .into_iter()
                    .map(|eco| view! { <EcosystemBadge ecosystem=eco/> })
                    .collect_view()}
            </div>

            <div class="project-card__stats">
                <div class="project-card__stat">
                    <span class="project-card__stat-value">{dependency_count}</span>
                    <span class="project-card__stat-label">"Dependencies"</span>
                </div>
                <div class="project-card__stat project-card__stat--alert">
                    <span class="project-card__stat-value">{alert_count}</span>
                    <span class="project-card__stat-label">"Open Alerts"</span>
                </div>
                {slsa_level.map(|level| view! {
                    <div class="project-card__stat">
                        <SlsaLevel level=Some(level)/>
                    </div>
                })}
            </div>

            <div class="project-card__footer">
                {last_scan.map(|scan| view! {
                    <span class="project-card__scan-time">"Last scan: " {scan}</span>
                })}
                <a href="#" class="project-card__link">
                    "View details"
                    <ArrowRightIcon/>
                </a>
            </div>

            <div class="project-card__geometric"></div>
        </article>
    }
}

/// Alert card with severity indicator and details.
#[component]
pub fn AlertCard(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(into)] severity: String,
    #[prop(into)] alert_type: String,
    #[prop(optional)] package_name: Option<String>,
    #[prop(optional)] ecosystem: Option<String>,
    #[prop(into)] created_at: String,
    #[prop(into)] status: String,
) -> impl IntoView {
    let severity_lower = severity.to_lowercase();
    let severity_level = match severity_lower.as_str() {
        "critical" | "high" => StatusLevel::Critical,
        "medium" => StatusLevel::Warning,
        "low" => StatusLevel::Info,
        _ => StatusLevel::Neutral,
    };

    view! {
        <article class=format!("alert-card alert-card--{severity_lower}")>
            <div class="alert-card__severity-bar"></div>

            <div class="alert-card__content">
                <div class="alert-card__header">
                    <SeverityTag severity=severity/>
                    <span class="alert-card__type">{alert_type}</span>
                    <span class="alert-card__time">{created_at}</span>
                </div>

                <h3 class="alert-card__title">{title}</h3>
                <p class="alert-card__description">{description}</p>

                {(package_name.is_some() || ecosystem.is_some()).then(|| view! {
                    <div class="alert-card__package">
                        <PackageIcon/>
                        <span class="alert-card__package-name">
                            {package_name.unwrap_or_default()}
                        </span>
                        {ecosystem.map(|eco| view! { <EcosystemBadge ecosystem=eco/> })}
                    </div>
                })}

                <div class="alert-card__footer">
                    <StatusBadge level=severity_level label=status/>
                    <div class="alert-card__actions">
                        <button class="alert-card__action">"Acknowledge"</button>
                        <button class="alert-card__action alert-card__action--primary">"Investigate"</button>
                    </div>
                </div>
            </div>
        </article>
    }
}

/// Policy card with rules summary.
#[component]
pub fn PolicyCard(
    #[prop(into)] name: String,
    #[prop(optional)] description: Option<String>,
    #[prop(into)] rule_count: u32,
    #[prop(into)] is_default: bool,
    #[prop(into)] enabled: bool,
    #[prop(into)] projects_count: u32,
) -> impl IntoView {
    view! {
        <article class=format!("policy-card {}", if enabled { "" } else { "policy-card--disabled" })>
            <div class="policy-card__header">
                <div class="policy-card__title-row">
                    <h3 class="policy-card__name">{name}</h3>
                    {is_default.then(|| view! {
                        <span class="policy-card__default-badge">"Default"</span>
                    })}
                </div>
                {description.map(|d| view! { <p class="policy-card__description">{d}</p> })}
            </div>

            <div class="policy-card__stats">
                <div class="policy-card__stat">
                    <span class="policy-card__stat-value">{rule_count}</span>
                    <span class="policy-card__stat-label">"Rules"</span>
                </div>
                <div class="policy-card__stat">
                    <span class="policy-card__stat-value">{projects_count}</span>
                    <span class="policy-card__stat-label">"Projects"</span>
                </div>
            </div>

            <div class="policy-card__footer">
                <span class=format!("policy-card__status {}", if enabled { "policy-card__status--enabled" } else { "policy-card__status--disabled" })>
                    {if enabled { "Enabled" } else { "Disabled" }}
                </span>
                <a href="#" class="policy-card__link">
                    "Configure"
                    <ArrowRightIcon/>
                </a>
            </div>

            <div class="policy-card__geometric"></div>
        </article>
    }
}

/// Rule item display for policy details.
#[component]
pub fn RuleItem(
    #[prop(into)] rule_type: String,
    #[prop(into)] description: String,
    #[prop(into)] severity: String,
    #[prop(into)] enabled: bool,
) -> impl IntoView {
    view! {
        <div class=format!("rule-item {}", if enabled { "" } else { "rule-item--disabled" })>
            <div class="rule-item__indicator">
                <StatusDot
                    level=if enabled { StatusLevel::Success } else { StatusLevel::Neutral }
                />
            </div>
            <div class="rule-item__content">
                <span class="rule-item__type">{rule_type}</span>
                <span class="rule-item__description">{description}</span>
            </div>
            <SeverityTag severity=severity/>
        </div>
    }
}

/// Dependency card for package details.
#[component]
pub fn DependencyCard(
    #[prop(into)] name: String,
    #[prop(into)] version: String,
    #[prop(into)] ecosystem: String,
    #[prop(into)] is_direct: bool,
    #[prop(optional)] alert_count: Option<u32>,
    #[prop(optional)] registry_url: Option<String>,
) -> impl IntoView {
    let has_alerts = alert_count.is_some_and(|c| c > 0);

    view! {
        <div class=format!("dependency-card {}", if has_alerts { "dependency-card--has-alerts" } else { "" })>
            <div class="dependency-card__header">
                <PackageIcon/>
                <span class="dependency-card__name">{name}</span>
                <span class="dependency-card__version">{version}</span>
            </div>

            <div class="dependency-card__meta">
                <EcosystemBadge ecosystem=ecosystem/>
                <span class=format!("dependency-card__type {}", if is_direct { "dependency-card__type--direct" } else { "" })>
                    {if is_direct { "Direct" } else { "Transitive" }}
                </span>
            </div>

            {alert_count.filter(|&c| c > 0).map(|count| view! {
                <div class="dependency-card__alerts">
                    <StatusDot level=StatusLevel::Warning/>
                    <span>{count} " alert" {if count > 1 { "s" } else { "" }}</span>
                </div>
            })}

            {registry_url.map(|url| view! {
                <a href=url class="dependency-card__link" target="_blank" rel="noopener">
                    "View on registry"
                    <ExternalLinkIcon/>
                </a>
            })}
        </div>
    }
}

/// Summary card for dashboard overview.
#[component]
pub fn SummaryCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] subtitle: String,
    #[prop(optional)] trend: Option<(f64, bool)>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    view! {
        <div class=format!("summary-card {}", class.unwrap_or_default())>
            <div class="summary-card__content">
                <span class="summary-card__title">{title}</span>
                <span class="summary-card__value">{value}</span>
                <span class="summary-card__subtitle">{subtitle}</span>
                {trend.map(|(change, positive)| view! {
                    <span class=format!("summary-card__trend {}", if positive { "summary-card__trend--up" } else { "summary-card__trend--down" })>
                        {if positive { "+" } else { "" }}{format!("{change:.1}%")}
                    </span>
                })}
            </div>
            <div class="summary-card__geometric"></div>
        </div>
    }
}
