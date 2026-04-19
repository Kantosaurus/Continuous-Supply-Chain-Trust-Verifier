//! Projects page - Dashboard overview and project management.
//!
//! Features an asymmetric grid layout with bold stats and
//! project cards arranged in a visually dynamic composition.

use leptos::prelude::*;

use crate::components::{
    AlertIcon, AsymmetricGrid, Button, ButtonVariant, GridItem, PackageIcon, PageHeader, PlusIcon,
    PolicyIcon, ProjectCard, SearchInput, Section, StatCard, StatsRow, StatusLevel,
};

/// Projects overview and management page.
#[component]
pub fn ProjectsPage() -> impl IntoView {
    let (search_query, _set_search_query) = signal(String::new());

    let mock_projects = vec![
        ProjectData {
            name: "payment-service".to_string(),
            description: Some("Core payment processing microservice".to_string()),
            status: StatusLevel::Success,
            dependency_count: 142,
            alert_count: 0,
            ecosystems: vec!["npm".to_string(), "cargo".to_string()],
            last_scan: Some("2 hours ago".to_string()),
            slsa_level: Some(3),
        },
        ProjectData {
            name: "auth-gateway".to_string(),
            description: Some("Authentication and authorization gateway".to_string()),
            status: StatusLevel::Warning,
            dependency_count: 89,
            alert_count: 3,
            ecosystems: vec!["npm".to_string()],
            last_scan: Some("4 hours ago".to_string()),
            slsa_level: Some(2),
        },
        ProjectData {
            name: "data-pipeline".to_string(),
            description: Some("ETL and data transformation pipeline".to_string()),
            status: StatusLevel::Critical,
            dependency_count: 256,
            alert_count: 7,
            ecosystems: vec!["pypi".to_string(), "cargo".to_string()],
            last_scan: Some("1 hour ago".to_string()),
            slsa_level: Some(1),
        },
        ProjectData {
            name: "api-gateway".to_string(),
            description: Some("Public API routing and rate limiting".to_string()),
            status: StatusLevel::Success,
            dependency_count: 67,
            alert_count: 0,
            ecosystems: vec!["cargo".to_string()],
            last_scan: Some("30 minutes ago".to_string()),
            slsa_level: Some(3),
        },
        ProjectData {
            name: "notification-worker".to_string(),
            description: Some("Async notification processing service".to_string()),
            status: StatusLevel::Warning,
            dependency_count: 45,
            alert_count: 1,
            ecosystems: vec!["npm".to_string()],
            last_scan: Some("6 hours ago".to_string()),
            slsa_level: None,
        },
    ];

    let filtered_projects = {
        let projects = mock_projects.clone();
        move || {
            let query = search_query.get().to_lowercase();
            if query.is_empty() {
                projects.clone()
            } else {
                projects
                    .iter()
                    .filter(|p| p.name.to_lowercase().contains(&query))
                    .cloned()
                    .collect()
            }
        }
    };

    view! {
        <div class="projects-page">
            <PageHeader
                title="Projects"
                subtitle="Monitor and manage your supply chain security".to_string()
            >
                <Button variant=ButtonVariant::Primary>
                    <PlusIcon/>
                    "Add Project"
                </Button>
            </PageHeader>

            <Section class="projects-page__stats".to_string()>
                <StatsRow>
                    <StatCard
                        value="5"
                        label="Total Projects"
                        class="stat-card--featured".to_string()
                    />
                    <StatCard
                        value="599"
                        label="Dependencies"
                        trend=(12.5, true)
                    />
                    <StatCard
                        value="11"
                        label="Open Alerts"
                        trend=(3.2, false)
                        class="stat-card--alert".to_string()
                    />
                    <StatCard value="2.4" label="Avg SLSA Level"/>
                </StatsRow>
            </Section>

            <Section class="projects-page__overview".to_string()>
                <div class="overview-grid">
                    <div class="overview-card overview-card--large">
                        <div class="overview-card__header">
                            <PackageIcon/>
                            <span>"Ecosystem Distribution"</span>
                        </div>
                        <div class="overview-card__chart">
                            <div class="ecosystem-bar">
                                <div class="ecosystem-bar__segment ecosystem-bar__segment--npm" style="width: 45%">
                                    <span>"npm"</span>
                                    <span>"45%"</span>
                                </div>
                                <div class="ecosystem-bar__segment ecosystem-bar__segment--cargo" style="width: 35%">
                                    <span>"cargo"</span>
                                    <span>"35%"</span>
                                </div>
                                <div class="ecosystem-bar__segment ecosystem-bar__segment--pypi" style="width: 20%">
                                    <span>"pypi"</span>
                                    <span>"20%"</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="overview-card">
                        <div class="overview-card__header">
                            <AlertIcon/>
                            <span>"Alert Breakdown"</span>
                        </div>
                        <div class="overview-card__metrics">
                            <div class="metric metric--critical">
                                <span class="metric__value">"2"</span>
                                <span class="metric__label">"Critical"</span>
                            </div>
                            <div class="metric metric--warning">
                                <span class="metric__value">"5"</span>
                                <span class="metric__label">"Warning"</span>
                            </div>
                            <div class="metric metric--info">
                                <span class="metric__value">"4"</span>
                                <span class="metric__label">"Info"</span>
                            </div>
                        </div>
                    </div>

                    <div class="overview-card">
                        <div class="overview-card__header">
                            <PolicyIcon/>
                            <span>"Policy Compliance"</span>
                        </div>
                        <div class="overview-card__compliance">
                            <div class="compliance-ring">
                                <svg viewBox="0 0 100 100">
                                    <circle cx="50" cy="50" r="40" class="compliance-ring__track"/>
                                    <circle
                                        cx="50"
                                        cy="50"
                                        r="40"
                                        class="compliance-ring__fill"
                                        stroke-dasharray="201"
                                        stroke-dashoffset="40"
                                    />
                                </svg>
                                <span class="compliance-ring__value">"80%"</span>
                            </div>
                            <span class="compliance-label">"4 of 5 projects compliant"</span>
                        </div>
                    </div>
                </div>
            </Section>

            <Section title="All Projects".to_string() class="projects-page__list".to_string()>
                <div class="projects-toolbar">
                    <SearchInput
                        placeholder="Search projects...".to_string()
                    />
                    <div class="projects-toolbar__filters">
                        <button class="filter-btn filter-btn--active">"All"</button>
                        <button class="filter-btn">"Healthy"</button>
                        <button class="filter-btn">"Warning"</button>
                        <button class="filter-btn">"Critical"</button>
                    </div>
                </div>

                <AsymmetricGrid class="projects-grid".to_string()>
                    {move || {
                        filtered_projects()
                            .into_iter()
                            .enumerate()
                            .map(|(i, project)| {
                                let col_span = if i == 0 { 2 } else { 1 };
                                view! {
                                    <GridItem col_span=col_span>
                                        <ProjectCard
                                            name=project.name
                                            description=project.description.unwrap_or_default()
                                            status=project.status
                                            dependency_count=project.dependency_count
                                            alert_count=project.alert_count
                                            ecosystems=project.ecosystems
                                            last_scan=project.last_scan.unwrap_or_default()
                                            slsa_level=project.slsa_level.unwrap_or(0)
                                        />
                                    </GridItem>
                                }
                            })
                            .collect_view()
                    }}
                </AsymmetricGrid>
            </Section>
        </div>
    }
}

#[derive(Clone)]
struct ProjectData {
    name: String,
    description: Option<String>,
    status: StatusLevel,
    dependency_count: u32,
    alert_count: u32,
    ecosystems: Vec<String>,
    last_scan: Option<String>,
    slsa_level: Option<u8>,
}
