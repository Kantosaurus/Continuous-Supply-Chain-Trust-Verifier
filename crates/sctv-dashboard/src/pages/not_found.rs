//! 404 Not Found page with bold geometric styling.

use leptos::prelude::*;

use crate::components::{ArrowRightIcon, Button, ButtonVariant};

/// 404 Not Found page.
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="not-found-page">
            <div class="not-found-page__content">
                <div class="not-found-page__geometric">
                    <span class="not-found-page__code">"404"</span>
                </div>
                <h1 class="not-found-page__title">"Page Not Found"</h1>
                <p class="not-found-page__description">
                    "The page you're looking for doesn't exist or has been moved."
                </p>
                <a href="/" class="not-found-page__link">
                    <Button variant=ButtonVariant::Primary>
                        "Back to Dashboard"
                        <ArrowRightIcon/>
                    </Button>
                </a>
            </div>
            <div class="not-found-page__bg-geometric"></div>
        </div>
    }
}
