//! Alerts page - Security alert management and triage.
//!
//! Features bold severity indicators, asymmetric card layouts,
//! and clear visual hierarchy for threat assessment.

use leptos::prelude::*;

use crate::components::{
    AlertCard, Button, ButtonVariant, FilterIcon, PageHeader, RefreshIcon, SearchInput,
    Section, StatCard, StatusIndicator, StatusLevel, StatsRow, TabNav,
};

/// Alerts management and triage page.
#[component]
pub fn AlertsPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("all".to_string());
    let (search_query, _set_search_query) = signal(String::new());

    let tabs = vec![
        ("all".to_string(), "All Alerts".to_string()),
        ("open".to_string(), "Open".to_string()),
        ("acknowledged".to_string(), "Acknowledged".to_string()),
        ("resolved".to_string(), "Resolved".to_string()),
    ];

    let mock_alerts = vec![
        AlertData {
            title: "Dependency tampering detected".to_string(),
            description: "Hash mismatch for package lodash@4.17.21. Expected sha256:abc123 but received sha256:def456.".to_string(),
            severity: "Critical".to_string(),
            alert_type: "Tampering".to_string(),
            package_name: Some("lodash".to_string()),
            ecosystem: Some("npm".to_string()),
            created_at: "2 hours ago".to_string(),
            status: "Open".to_string(),
        },
        AlertData {
            title: "Potential typosquatting attack".to_string(),
            description: "Package 'react-dom-utils' is suspiciously similar to popular package 'react-dom'. Similarity score: 0.92.".to_string(),
            severity: "Critical".to_string(),
            alert_type: "Typosquatting".to_string(),
            package_name: Some("react-dom-utils".to_string()),
            ecosystem: Some("npm".to_string()),
            created_at: "4 hours ago".to_string(),
            status: "Investigating".to_string(),
        },
        AlertData {
            title: "Version downgrade detected".to_string(),
            description: "Package 'serde' was downgraded from 1.0.200 to 1.0.150. This may indicate a rollback attack.".to_string(),
            severity: "High".to_string(),
            alert_type: "Downgrade".to_string(),
            package_name: Some("serde".to_string()),
            ecosystem: Some("cargo".to_string()),
            created_at: "6 hours ago".to_string(),
            status: "Acknowledged".to_string(),
        },
        AlertData {
            title: "SLSA provenance verification failed".to_string(),
            description: "Package 'requests' version 2.31.0 does not meet the required SLSA level 2. Current level: 0.".to_string(),
            severity: "Medium".to_string(),
            alert_type: "Provenance".to_string(),
            package_name: Some("requests".to_string()),
            ecosystem: Some("pypi".to_string()),
            created_at: "1 day ago".to_string(),
            status: "Open".to_string(),
        },
        AlertData {
            title: "Policy violation: Unpinned dependency".to_string(),
            description: "Dependency 'axios' uses range specifier '^1.0.0' which violates the 'Exact Version Pinning' policy rule.".to_string(),
            severity: "Medium".to_string(),
            alert_type: "Policy".to_string(),
            package_name: Some("axios".to_string()),
            ecosystem: Some("npm".to_string()),
            created_at: "1 day ago".to_string(),
            status: "Open".to_string(),
        },
        AlertData {
            title: "New package with low age".to_string(),
            description: "Package 'fast-json-parser' was published only 3 days ago. Minimum age requirement: 30 days.".to_string(),
            severity: "Low".to_string(),
            alert_type: "New Package".to_string(),
            package_name: Some("fast-json-parser".to_string()),
            ecosystem: Some("npm".to_string()),
            created_at: "2 days ago".to_string(),
            status: "Open".to_string(),
        },
    ];

    let filtered_alerts = {
        let alerts = mock_alerts.clone();
        move || {
            let tab = active_tab.get();
            let query = search_query.get().to_lowercase();

            alerts
                .iter()
                .filter(|a| {
                    let tab_match = match tab.as_str() {
                        "open" => a.status == "Open" || a.status == "Investigating",
                        "acknowledged" => a.status == "Acknowledged",
                        "resolved" => a.status == "Resolved",
                        _ => true,
                    };
                    let search_match = query.is_empty()
                        || a.title.to_lowercase().contains(&query)
                        || a.package_name
                            .as_ref()
                            .map_or(false, |p| p.to_lowercase().contains(&query));
                    tab_match && search_match
                })
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    view! {
        <div class="alerts-page">
            <PageHeader
                title="Alerts"
                subtitle="Security findings and threat detection".to_string()
            >
                <Button variant=ButtonVariant::Ghost>
                    <RefreshIcon/>
                    "Refresh"
                </Button>
            </PageHeader>

            <Section class="alerts-page__summary".to_string()>
                <div class="alerts-summary">
                    <StatusIndicator
                        level=StatusLevel::Critical
                        count=2u32
                        label="Critical"
                    />
                    <StatusIndicator level=StatusLevel::Warning count=3u32 label="Warning"/>
                    <StatusIndicator level=StatusLevel::Info count=6u32 label="Info"/>
                    <StatusIndicator level=StatusLevel::Success count=12u32 label="Resolved"/>
                </div>
            </Section>

            <Section class="alerts-page__metrics".to_string()>
                <StatsRow>
                    <StatCard value="23" label="Total Alerts"/>
                    <StatCard
                        value="11"
                        label="Open"
                        trend=(15.0, false)
                        class="stat-card--warning".to_string()
                    />
                    <StatCard value="4.2h" label="Avg Resolution Time"/>
                    <StatCard value="89%" label="SLA Compliance" trend=(2.1, true)/>
                </StatsRow>
            </Section>

            <Section class="alerts-page__list".to_string()>
                <div class="alerts-toolbar">
                    <TabNav tabs=tabs active_tab=active_tab on_change=move |t| set_active_tab.set(t)/>

                    <div class="alerts-toolbar__actions">
                        <SearchInput
                            placeholder="Search alerts...".to_string()
                        />
                        <Button variant=ButtonVariant::Ghost>
                            <FilterIcon/>
                            "Filter"
                        </Button>
                    </div>
                </div>

                <div class="alerts-list">
                    {move || {
                        let alerts = filtered_alerts();
                        if alerts.is_empty() {
                            view! {
                                <div class="alerts-empty">
                                    <div class="alerts-empty__geometric"></div>
                                    <h3>"No alerts found"</h3>
                                    <p>"Try adjusting your filters or search query."</p>
                                </div>
                            }
                                .into_any()
                        } else {
                            alerts
                                .into_iter()
                                .map(|alert| {
                                    view! {
                                        <AlertCard
                                            title=alert.title
                                            description=alert.description
                                            severity=alert.severity
                                            alert_type=alert.alert_type
                                            package_name=alert.package_name.unwrap_or_default()
                                            ecosystem=alert.ecosystem.unwrap_or_default()
                                            created_at=alert.created_at
                                            status=alert.status
                                        />
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </div>
            </Section>
        </div>
    }
}

#[derive(Clone)]
struct AlertData {
    title: String,
    description: String,
    severity: String,
    alert_type: String,
    package_name: Option<String>,
    ecosystem: Option<String>,
    created_at: String,
    status: String,
}
