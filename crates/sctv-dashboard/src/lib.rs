//! SCTV Dashboard - Leptos Web UI for Supply Chain Trust Verifier
//!
//! A clean, bold, and asymmetric dashboard with an innovative white-dominant design.

pub mod components;
pub mod pages;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Meta, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use components::layout::MainLayout;
use pages::{AlertsPage, NotFoundPage, PoliciesPage, ProjectsPage, SettingsPage};

/// Main application component with routing setup.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/sctv-dashboard.css"/>
        <Link rel="preconnect" href="https://fonts.googleapis.com"/>
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous"/>
        <Link
            href="https://fonts.googleapis.com/css2?family=Instrument+Sans:wght@400;500;600;700&family=Space+Mono:wght@400;700&display=swap"
            rel="stylesheet"
        />
        <Title text="SCTV | Supply Chain Trust Verifier"/>
        <Meta name="description" content="Supply Chain Trust Verifier - Secure your software supply chain"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Router>
            <MainLayout>
                <Routes fallback=|| view! { <NotFoundPage/> }>
                    <Route path=path!("/") view=ProjectsPage/>
                    <Route path=path!("/projects") view=ProjectsPage/>
                    <Route path=path!("/alerts") view=AlertsPage/>
                    <Route path=path!("/policies") view=PoliciesPage/>
                    <Route path=path!("/settings") view=SettingsPage/>
                </Routes>
            </MainLayout>
        </Router>
    }
}

/// Hydrate the app for client-side rendering.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

/// Mount the app for CSR-only mode.
#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
