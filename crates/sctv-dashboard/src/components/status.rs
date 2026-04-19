//! Status indicators and badges for visual feedback.
//!
//! These components provide bold, high-contrast status
//! visualization that stands out in the white-dominant interface.

use leptos::prelude::*;

/// Status severity level for visual styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusLevel {
    #[default]
    Neutral,
    Success,
    Warning,
    Critical,
    Info,
}

impl StatusLevel {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Neutral => "status--neutral",
            Self::Success => "status--success",
            Self::Warning => "status--warning",
            Self::Critical => "status--critical",
            Self::Info => "status--info",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Neutral => "Unknown",
            Self::Success => "Healthy",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
            Self::Info => "Info",
        }
    }
}

/// A bold status badge with floating geometric accent.
#[component]
pub fn StatusBadge(
    #[prop(into)] level: StatusLevel,
    #[prop(optional)] label: Option<String>,
) -> impl IntoView {
    let display_label = label.unwrap_or_else(|| level.label().to_string());

    view! {
        <span class=format!("status-badge {}", level.class())>
            <span class="status-badge__indicator"></span>
            <span class="status-badge__label">{display_label}</span>
        </span>
    }
}

/// A compact status dot for inline use.
#[component]
pub fn StatusDot(#[prop(into)] level: StatusLevel) -> impl IntoView {
    view! {
        <span class=format!("status-dot {}", level.class()) title=level.label()></span>
    }
}

/// Large status indicator for dashboard cards.
#[component]
pub fn StatusIndicator(
    #[prop(into)] level: StatusLevel,
    #[prop(into)] count: u32,
    #[prop(into)] label: String,
) -> impl IntoView {
    view! {
        <div class=format!("status-indicator {}", level.class())>
            <div class="status-indicator__geometric"></div>
            <span class="status-indicator__count">{count}</span>
            <span class="status-indicator__label">{label}</span>
        </div>
    }
}

/// Progress bar with asymmetric styling.
#[component]
pub fn ProgressBar(
    #[prop(into)] value: f64,
    #[prop(optional)] level: StatusLevel,
) -> impl IntoView {
    let percentage = (value.clamp(0.0, 100.0) * 100.0).round() / 100.0;

    view! {
        <div class=format!("progress-bar {}", level.class())>
            <div class="progress-bar__track">
                <div
                    class="progress-bar__fill"
                    style=format!("width: {}%", percentage)
                ></div>
            </div>
            <span class="progress-bar__value">{format!("{:.0}%", percentage)}</span>
        </div>
    }
}

/// Trust score display with bold typography.
#[component]
pub fn TrustScore(#[prop(into)] score: u8) -> impl IntoView {
    let level = match score {
        0..=39 => StatusLevel::Critical,
        40..=69 => StatusLevel::Warning,
        70..=100 => StatusLevel::Success,
        _ => StatusLevel::Neutral,
    };

    view! {
        <div class=format!("trust-score {}", level.class())>
            <span class="trust-score__value">{score}</span>
            <span class="trust-score__label">"Trust Score"</span>
        </div>
    }
}

/// Severity tag for alert types.
#[component]
pub fn SeverityTag(#[prop(into)] severity: String) -> impl IntoView {
    let level = match severity.to_lowercase().as_str() {
        "critical" => StatusLevel::Critical,
        "high" => StatusLevel::Critical,
        "medium" => StatusLevel::Warning,
        "low" => StatusLevel::Info,
        _ => StatusLevel::Neutral,
    };

    view! {
        <span class=format!("severity-tag {}", level.class())>
            {severity}
        </span>
    }
}

/// Ecosystem badge showing package manager.
#[component]
pub fn EcosystemBadge(#[prop(into)] ecosystem: String) -> impl IntoView {
    let eco_lower = ecosystem.to_lowercase();
    view! {
        <span class="ecosystem-badge" data-ecosystem=eco_lower>
            {ecosystem}
        </span>
    }
}

/// SLSA level indicator.
#[component]
pub fn SlsaLevel(#[prop(into)] level: Option<u8>) -> impl IntoView {
    let display = level
        .map(|l| format!("L{}", l))
        .unwrap_or_else(|| "—".to_string());
    let status = match level {
        Some(3..=4) => StatusLevel::Success,
        Some(2) => StatusLevel::Info,
        Some(1) => StatusLevel::Warning,
        _ => StatusLevel::Neutral,
    };

    view! {
        <div class=format!("slsa-level {}", status.class())>
            <span class="slsa-level__badge">"SLSA"</span>
            <span class="slsa-level__value">{display}</span>
        </div>
    }
}

/// Activity pulse animation for real-time status.
#[component]
pub fn ActivityPulse(#[prop(optional)] active: bool) -> impl IntoView {
    view! {
        <span class=if active { "activity-pulse activity-pulse--active" } else { "activity-pulse" }>
            <span class="activity-pulse__ring"></span>
            <span class="activity-pulse__core"></span>
        </span>
    }
}

/// Count badge for notifications/alerts.
#[component]
pub fn CountBadge(#[prop(into)] count: u32) -> impl IntoView {
    let display = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };
    let visible = count > 0;

    view! {
        <span class="count-badge" class:count-badge--hidden=!visible>
            {display}
        </span>
    }
}
