//! SCTV Dashboard Server
//!
//! This binary provides the SSR server for the SCTV Dashboard.
//! For CSR-only mode, use the WASM build.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sctv_dashboard::App;
    use tower_http::services::ServeDir;

    tracing_subscriber::fmt::init();

    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .nest_service("/assets", ServeDir::new("public"))
        .with_state(leptos_options);

    log!("Starting SCTV Dashboard at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(feature = "ssr")]
fn shell(options: leptos::prelude::LeptosOptions) -> impl leptos::IntoView {
    use leptos::prelude::*;
    use leptos_meta::*;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <sctv_dashboard::App/>
            </body>
        </html>
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // This main function is a placeholder for non-SSR builds.
    // The actual entry point for CSR is in lib.rs via wasm-bindgen.
    println!("This binary requires the 'ssr' feature. For CSR, build the WASM target.");
}
