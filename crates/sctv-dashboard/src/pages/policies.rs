//! Policies page - Security policy configuration and management.
//!
//! Features clean rule displays with bold typography and
//! geometric accents for policy visualization.

use leptos::prelude::*;

use crate::components::{
    AsymmetricGrid, Button, ButtonVariant, Divider, GridItem, PageHeader, PlusIcon, PolicyCard,
    RuleItem, Section, StatCard, StatsRow, Toggle,
};

/// Policy management page.
#[component]
pub fn PoliciesPage() -> impl IntoView {
    let (selected_policy, set_selected_policy) = signal::<Option<usize>>(Some(0));

    let mock_policies = vec![
        PolicyData {
            name: "Strict Security".to_string(),
            description: Some("Comprehensive security checks for production services".to_string()),
            rule_count: 8,
            is_default: true,
            enabled: true,
            projects_count: 3,
            rules: vec![
                RuleData {
                    rule_type: "Hash Verification".to_string(),
                    description: "Require SHA-256 hash verification for all packages".to_string(),
                    severity: "High".to_string(),
                    enabled: true,
                },
                RuleData {
                    rule_type: "Block Typosquatting".to_string(),
                    description: "Block packages with >85% similarity to popular packages".to_string(),
                    severity: "Critical".to_string(),
                    enabled: true,
                },
                RuleData {
                    rule_type: "Require Provenance".to_string(),
                    description: "Require minimum SLSA level 1 for all dependencies".to_string(),
                    severity: "Medium".to_string(),
                    enabled: true,
                },
                RuleData {
                    rule_type: "Version Pinning".to_string(),
                    description: "Enforce locked version pinning strategy".to_string(),
                    severity: "Medium".to_string(),
                    enabled: true,
                },
                RuleData {
                    rule_type: "Minimum Age".to_string(),
                    description: "Require packages to be at least 30 days old".to_string(),
                    severity: "Low".to_string(),
                    enabled: false,
                },
            ],
        },
        PolicyData {
            name: "Moderate".to_string(),
            description: Some("Balanced security for staging environments".to_string()),
            rule_count: 4,
            is_default: false,
            enabled: true,
            projects_count: 1,
            rules: vec![
                RuleData {
                    rule_type: "Hash Verification".to_string(),
                    description: "Require SHA-256 hash verification".to_string(),
                    severity: "High".to_string(),
                    enabled: true,
                },
                RuleData {
                    rule_type: "Block Typosquatting".to_string(),
                    description: "Block packages with >90% similarity".to_string(),
                    severity: "Critical".to_string(),
                    enabled: true,
                },
            ],
        },
        PolicyData {
            name: "Permissive".to_string(),
            description: Some("Minimal checks for development environments".to_string()),
            rule_count: 2,
            is_default: false,
            enabled: true,
            projects_count: 1,
            rules: vec![RuleData {
                rule_type: "Block Typosquatting".to_string(),
                description: "Block packages with >95% similarity only".to_string(),
                severity: "Critical".to_string(),
                enabled: true,
            }],
        },
        PolicyData {
            name: "Audit Only".to_string(),
            description: Some("Logging and monitoring without blocking".to_string()),
            rule_count: 0,
            is_default: false,
            enabled: false,
            projects_count: 0,
            rules: vec![],
        },
    ];

    let current_policy = {
        let policies = mock_policies.clone();
        move || {
            selected_policy
                .get()
                .and_then(|idx| policies.get(idx).cloned())
        }
    };

    view! {
        <div class="policies-page">
            <PageHeader
                title="Policies"
                subtitle="Configure security rules and enforcement".to_string()
            >
                <Button variant=ButtonVariant::Primary>
                    <PlusIcon/>
                    "New Policy"
                </Button>
            </PageHeader>

            <Section class="policies-page__stats".to_string()>
                <StatsRow>
                    <StatCard value="4" label="Total Policies"/>
                    <StatCard value="3" label="Active"/>
                    <StatCard value="14" label="Total Rules"/>
                    <StatCard value="80%" label="Project Coverage"/>
                </StatsRow>
            </Section>

            <div class="policies-layout">
                <Section class="policies-page__list".to_string()>
                    <h2 class="section__title">"All Policies"</h2>
                    <AsymmetricGrid class="policies-grid".to_string()>
                        {mock_policies
                            .clone()
                            .into_iter()
                            .enumerate()
                            .map(|(idx, policy)| {
                                let is_selected = move || selected_policy.get() == Some(idx);
                                view! {
                                    <GridItem class=if is_selected() {
                                        "policy-item policy-item--selected".to_string()
                                    } else {
                                        "policy-item".to_string()
                                    }>
                                        <div
                                            class="policy-item__wrapper"
                                            on:click=move |_| set_selected_policy.set(Some(idx))
                                        >
                                            <PolicyCard
                                                name=policy.name
                                                description=policy.description.unwrap_or_default()
                                                rule_count=policy.rule_count
                                                is_default=policy.is_default
                                                enabled=policy.enabled
                                                projects_count=policy.projects_count
                                            />
                                        </div>
                                    </GridItem>
                                }
                            })
                            .collect_view()}
                    </AsymmetricGrid>
                </Section>

                <Section class="policies-page__detail".to_string()>
                    {move || {
                        if let Some(policy) = current_policy() {
                            view! {
                                <div class="policy-detail">
                                    <div class="policy-detail__header">
                                        <div class="policy-detail__title-group">
                                            <h2 class="policy-detail__name">{policy.name.clone()}</h2>
                                            {policy.is_default.then(|| view! {
                                                <span class="policy-detail__default">"Default"</span>
                                            })}
                                        </div>
                                        <Toggle
                                            name="policy-enabled"
                                            label="Enabled"
                                            checked=policy.enabled
                                        />
                                    </div>

                                    {policy.description.map(|d| view! {
                                        <p class="policy-detail__description">{d}</p>
                                    })}

                                    <Divider label="Rules".to_string()/>

                                    <div class="policy-detail__rules">
                                        {if policy.rules.is_empty() {
                                            view! {
                                                <div class="policy-detail__empty">
                                                    <p>"No rules configured for this policy."</p>
                                                    <Button variant=ButtonVariant::Secondary>
                                                        <PlusIcon/>
                                                        "Add Rule"
                                                    </Button>
                                                </div>
                                            }
                                                .into_any()
                                        } else {
                                            policy
                                                .rules
                                                .into_iter()
                                                .map(|rule| {
                                                    view! {
                                                        <RuleItem
                                                            rule_type=rule.rule_type
                                                            description=rule.description
                                                            severity=rule.severity
                                                            enabled=rule.enabled
                                                        />
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }}
                                    </div>

                                    <div class="policy-detail__actions">
                                        <Button variant=ButtonVariant::Secondary>
                                            <PlusIcon/>
                                            "Add Rule"
                                        </Button>
                                        <Button variant=ButtonVariant::Ghost>"Duplicate Policy"</Button>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="policy-detail policy-detail--empty">
                                    <div class="policy-detail__placeholder">
                                        <p>"Select a policy to view details"</p>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </Section>
            </div>
        </div>
    }
}

#[derive(Clone)]
struct PolicyData {
    name: String,
    description: Option<String>,
    rule_count: u32,
    is_default: bool,
    enabled: bool,
    projects_count: u32,
    rules: Vec<RuleData>,
}

#[derive(Clone)]
struct RuleData {
    rule_type: String,
    description: String,
    severity: String,
    enabled: bool,
}
