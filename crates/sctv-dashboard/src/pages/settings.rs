//! Settings page - Application configuration and preferences.
//!
//! Features modern form design with clean sections and
//! bold typography for configuration management.

use leptos::prelude::*;

use crate::components::{
    Button, ButtonVariant, Checkbox, Divider, FieldGroup, FormActions, PageHeader, Section, Select,
    TabNav, TextInput, Textarea, Toggle,
};

/// Application settings page.
#[component]
pub fn SettingsPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("general".to_string());

    let tabs = vec![
        ("general".to_string(), "General".to_string()),
        ("notifications".to_string(), "Notifications".to_string()),
        ("integrations".to_string(), "Integrations".to_string()),
        ("security".to_string(), "Security".to_string()),
    ];

    view! {
        <div class="settings-page">
            <PageHeader
                title="Settings"
                subtitle="Configure your SCTV instance".to_string()
            />

            <div class="settings-layout">
                <aside class="settings-nav">
                    <TabNav
                        tabs=tabs
                        active_tab=active_tab
                        on_change=move |t| set_active_tab.set(t)
                    />
                </aside>

                <main class="settings-content">
                    {move || match active_tab.get().as_str() {
                        "notifications" => view! { <NotificationSettings/> }.into_any(),
                        "integrations" => view! { <IntegrationSettings/> }.into_any(),
                        "security" => view! { <SecuritySettings/> }.into_any(),
                        _ => view! { <GeneralSettings/> }.into_any(),
                    }}
                </main>
            </div>
        </div>
    }
}

#[component]
fn GeneralSettings() -> impl IntoView {
    view! {
        <div class="settings-section">
            <Section title="Organization".to_string()>
                <div class="settings-form">
                    <TextInput
                        name="org-name"
                        label="Organization Name"
                        value="Acme Corp".to_string()
                    />
                    <TextInput
                        name="org-slug"
                        label="Organization Slug"
                        value="acme-corp".to_string()
                    />
                    <Textarea
                        name="org-description"
                        label="Description"
                        placeholder="Brief description of your organization...".to_string()
                        rows=3
                    />
                </div>
            </Section>

            <Divider/>

            <Section title="Scanning Defaults".to_string()>
                <div class="settings-form">
                    <Select
                        name="default-schedule"
                        label="Default Scan Schedule"
                        options=vec![
                            ("hourly".to_string(), "Hourly".to_string()),
                            ("daily".to_string(), "Daily (Recommended)".to_string()),
                            ("weekly".to_string(), "Weekly".to_string()),
                            ("manual".to_string(), "Manual Only".to_string()),
                        ]
                        value="daily".to_string()
                    />
                    <Select
                        name="default-policy"
                        label="Default Policy"
                        options=vec![
                            ("strict".to_string(), "Strict Security".to_string()),
                            ("moderate".to_string(), "Moderate".to_string()),
                            ("permissive".to_string(), "Permissive".to_string()),
                        ]
                        value="strict".to_string()
                    />
                    <FieldGroup label="Ecosystems".to_string()>
                        <Checkbox name="eco-npm" label="npm" checked=true/>
                        <Checkbox name="eco-cargo" label="Cargo" checked=true/>
                        <Checkbox name="eco-pypi" label="PyPI" checked=true/>
                        <Checkbox name="eco-maven" label="Maven" checked=false/>
                        <Checkbox name="eco-nuget" label="NuGet" checked=false/>
                        <Checkbox name="eco-rubygems" label="RubyGems" checked=false/>
                    </FieldGroup>
                </div>
            </Section>

            <FormActions>
                <Button variant=ButtonVariant::Ghost>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary>"Save Changes"</Button>
            </FormActions>
        </div>
    }
}

#[component]
fn NotificationSettings() -> impl IntoView {
    view! {
        <div class="settings-section">
            <Section title="Alert Notifications".to_string()>
                <div class="settings-form">
                    <Toggle
                        name="notify-critical"
                        label="Notify on critical alerts"
                        checked=true
                    />
                    <Toggle
                        name="notify-high"
                        label="Notify on high severity alerts"
                        checked=true
                    />
                    <Toggle
                        name="notify-medium"
                        label="Notify on medium severity alerts"
                        checked=false
                    />
                    <Toggle
                        name="notify-low"
                        label="Notify on low severity alerts"
                        checked=false
                    />
                </div>
            </Section>

            <Divider/>

            <Section title="Email Notifications".to_string()>
                <div class="settings-form">
                    <Toggle name="email-enabled" label="Enable email notifications" checked=true/>
                    <TextInput
                        name="email-recipients"
                        label="Recipients"
                        placeholder="email@example.com, team@example.com".to_string()
                        value="security@acme.com".to_string()
                    />
                    <Select
                        name="email-digest"
                        label="Digest Frequency"
                        options=vec![
                            ("immediate".to_string(), "Immediate".to_string()),
                            ("hourly".to_string(), "Hourly Digest".to_string()),
                            ("daily".to_string(), "Daily Digest".to_string()),
                        ]
                        value="immediate".to_string()
                    />
                </div>
            </Section>

            <Divider/>

            <Section title="Slack Integration".to_string()>
                <div class="settings-form">
                    <Toggle name="slack-enabled" label="Enable Slack notifications" checked=false/>
                    <TextInput
                        name="slack-webhook"
                        label="Webhook URL"
                        placeholder="https://hooks.slack.com/services/...".to_string()
                    />
                    <TextInput
                        name="slack-channel"
                        label="Channel"
                        placeholder="#security-alerts".to_string()
                    />
                </div>
            </Section>

            <FormActions>
                <Button variant=ButtonVariant::Ghost>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary>"Save Changes"</Button>
            </FormActions>
        </div>
    }
}

#[component]
fn IntegrationSettings() -> impl IntoView {
    view! {
        <div class="settings-section">
            <Section title="CI/CD Integration".to_string()>
                <div class="settings-form">
                    <div class="integration-card">
                        <div class="integration-card__header">
                            <span class="integration-card__name">"GitHub Actions"</span>
                            <span class="integration-card__status integration-card__status--connected">
                                "Connected"
                            </span>
                        </div>
                        <p class="integration-card__description">
                            "Run SCTV checks as part of your GitHub Actions workflows."
                        </p>
                        <Button variant=ButtonVariant::Secondary>"Configure"</Button>
                    </div>

                    <div class="integration-card">
                        <div class="integration-card__header">
                            <span class="integration-card__name">"GitLab CI"</span>
                            <span class="integration-card__status">"Not Connected"</span>
                        </div>
                        <p class="integration-card__description">
                            "Integrate SCTV with your GitLab CI/CD pipelines."
                        </p>
                        <Button variant=ButtonVariant::Secondary>"Connect"</Button>
                    </div>

                    <div class="integration-card">
                        <div class="integration-card__header">
                            <span class="integration-card__name">"Jenkins"</span>
                            <span class="integration-card__status">"Not Connected"</span>
                        </div>
                        <p class="integration-card__description">
                            "Add SCTV verification to your Jenkins builds."
                        </p>
                        <Button variant=ButtonVariant::Secondary>"Connect"</Button>
                    </div>
                </div>
            </Section>

            <Divider/>

            <Section title="API Access".to_string()>
                <div class="settings-form">
                    <div class="api-key-section">
                        <TextInput
                            name="api-key"
                            label="API Key"
                            value="sctv_sk_••••••••••••••••••••".to_string()
                            disabled=true
                        />
                        <div class="api-key-actions">
                            <Button variant=ButtonVariant::Ghost>"Reveal"</Button>
                            <Button variant=ButtonVariant::Secondary>"Regenerate"</Button>
                        </div>
                    </div>
                    <p class="settings-hint">
                        "Use this API key to authenticate CLI and CI/CD integrations."
                    </p>
                </div>
            </Section>
        </div>
    }
}

#[component]
fn SecuritySettings() -> impl IntoView {
    view! {
        <div class="settings-section">
            <Section title="Authentication".to_string()>
                <div class="settings-form">
                    <Toggle name="mfa-enabled" label="Require multi-factor authentication" checked=true/>
                    <Toggle
                        name="sso-enabled"
                        label="Enable Single Sign-On (SSO)"
                        checked=false
                    />
                    <Select
                        name="session-timeout"
                        label="Session Timeout"
                        options=vec![
                            ("1h".to_string(), "1 hour".to_string()),
                            ("8h".to_string(), "8 hours".to_string()),
                            ("24h".to_string(), "24 hours".to_string()),
                            ("7d".to_string(), "7 days".to_string()),
                        ]
                        value="8h".to_string()
                    />
                </div>
            </Section>

            <Divider/>

            <Section title="Audit Logging".to_string()>
                <div class="settings-form">
                    <Toggle name="audit-enabled" label="Enable audit logging" checked=true/>
                    <Select
                        name="audit-retention"
                        label="Log Retention Period"
                        options=vec![
                            ("30d".to_string(), "30 days".to_string()),
                            ("90d".to_string(), "90 days".to_string()),
                            ("1y".to_string(), "1 year".to_string()),
                            ("forever".to_string(), "Forever".to_string()),
                        ]
                        value="90d".to_string()
                    />
                    <Toggle
                        name="audit-export"
                        label="Export audit logs to external SIEM"
                        checked=false
                    />
                </div>
            </Section>

            <Divider/>

            <Section title="Data Privacy".to_string()>
                <div class="settings-form">
                    <Toggle
                        name="anonymize-deps"
                        label="Anonymize dependency data in reports"
                        checked=false
                    />
                    <Toggle
                        name="local-only"
                        label="Keep all data on-premises"
                        checked=true
                    />
                </div>
            </Section>

            <FormActions>
                <Button variant=ButtonVariant::Ghost>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary>"Save Changes"</Button>
            </FormActions>
        </div>
    }
}
