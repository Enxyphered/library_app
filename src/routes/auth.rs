use serde::Deserialize;
use topcoat::{
    asset::asset,
    context::{app_context, Cx},
    cookie::{cookies, Cookie, Cookies},
    router::{content::Form, error::see_other, page, response::response_headers, route},
    view::{view, View, ViewExt, ViewHandle},
    Error, Result,
};

use crate::auth::{
    jwt::{create_jwt, AUTH_COOKIE_NAME, JWT_EXPIRATION_HOURS},
    password::verify_password,
};
use crate::components::icons::{
    icon_alert_triangle, icon_library, icon_lock, icon_moon, icon_sun, icon_user,
};
use crate::db::DbPool;

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[page("/login")]
pub async fn login_page(cx: &Cx) -> Result<impl View> {
    // If user is already authenticated, redirect to /
    if crate::auth::guard::current_user(cx).is_some() {
        return Err(see_other("/").into());
    }

    let is_dark = cookies(cx).get("theme").map(|c| c.value() == "dark").unwrap_or(false);
    let html_class = if is_dark { "dark" } else { "" };

    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" class=(html_class)>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>"Login | Library Management System"</title>
                <link rel="stylesheet" href=(topcoat::tailwind::stylesheet!())>
                <script src=(asset!("static/htmx.min.js")) defer="defer"></script>
                <script src=(asset!("static/theme-init.js"))></script>
                <script src=(asset!("static/app.js")) defer="defer"></script>
            </head>
            <body class="bg-background text-foreground antialiased min-h-screen flex items-center justify-center p-4 relative">
                // Theme Toggle in top-right corner
                <div class="fixed top-4 right-4 z-50">
                    <button
                        data-controller="theme"
                        type="button"
                        class="p-2.5 rounded-lg border border-border bg-card text-muted-foreground hover:text-foreground hover:bg-muted shadow-xs transition-colors cursor-pointer"
                        title="Toggle theme"
                        aria-label="Toggle theme"
                    >
                        <span class="dark:hidden">
                            icon_moon(class: "size-4 shrink-0")
                        </span>
                        <span class="hidden dark:inline">
                            icon_sun(class: "size-4 shrink-0")
                        </span>
                    </button>
                </div>

                <div class="w-full max-w-md bg-card text-card-foreground border border-border rounded-xl shadow-lg p-8 space-y-6">
                    <div class="flex flex-col items-center text-center space-y-2">
                        <div class="size-12 rounded-xl bg-primary text-primary-foreground flex items-center justify-center shadow-md shrink-0">
                            icon_library(class: "size-6")
                        </div>
                        <h1 class="text-2xl font-bold tracking-tight">"Library Portal"</h1>
                        <p class="text-sm text-muted-foreground">"Sign in with your librarian credentials"</p>
                    </div>

                    // Alert target for HTMX inline errors
                    <div id="login-alert"></div>

                    <form
                        method="POST"
                        action="/login"
                        hx-post="/login"
                        hx-target="#login-alert"
                        hx-swap="innerHTML"
                        class="space-y-4"
                    >
                        <div class="space-y-1.5">
                            <label for="username" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                "Username"
                            </label>
                            <div class="relative">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-muted-foreground">
                                    icon_user(class: "size-4 shrink-0")
                                </div>
                                <input
                                    id="username"
                                    name="username"
                                    type="text"
                                    required="required"
                                    placeholder="admin"
                                    value="admin"
                                    class="flex h-10 w-full rounded-md border border-input bg-background pl-9 pr-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                                />
                            </div>
                        </div>

                        <div class="space-y-1.5">
                            <label for="password" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                "Password"
                            </label>
                            <div class="relative">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-muted-foreground">
                                    icon_lock(class: "size-4 shrink-0")
                                </div>
                                <input
                                    id="password"
                                    name="password"
                                    type="password"
                                    required="required"
                                    placeholder="admin"
                                    value="admin"
                                    class="flex h-10 w-full rounded-md border border-input bg-background pl-9 pr-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                                />
                            </div>
                        </div>

                        <button
                            type="submit"
                            class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors bg-primary text-primary-foreground shadow hover:bg-primary/90 h-10 px-4 py-2 w-full font-semibold cursor-pointer"
                        >
                            "Sign In"
                        </button>
                    </form>

                    <div class="text-xs text-center text-muted-foreground bg-muted/40 p-3 rounded-lg border border-border">
                        <span class="font-medium text-foreground">"Default Seed Credentials: "</span>
                        <code class="font-mono bg-muted px-1.5 py-0.5 rounded text-foreground">"admin"</code>
                        " / "
                        <code class="font-mono bg-muted px-1.5 py-0.5 rounded text-foreground">"admin"</code>
                    </div>
                </div>
            </body>
        </html>
    })
}

#[route(POST "/login")]
pub async fn login_action(
    cx: &Cx,
    Form(form): Form<LoginForm>,
) -> Result<ViewHandle> {
    let pool: &DbPool = app_context(cx);

    let username_clean = form.username.trim().to_lowercase();
    let user_opt = {
        let pool = pool.clone();
        let u_clone = username_clean.clone();
        pool.read(move |conn| crate::db::queries::get_user_by_username(conn, &u_clone))
            .await
            .map_err(|e| Error::msg(e.to_string()))?
    };

    if let Some(user) = user_opt {
        if verify_password(&form.password, &user.password_hash) {
            let token = create_jwt(user.id, &user.username, user.role)
                .map_err(|e| Error::msg(e))?;

            let cookie = Cookie::build((AUTH_COOKIE_NAME, token))
                .path("/")
                .http_only(true)
                .same_site(topcoat::cookie::SameSite::Lax)
                .max_age(topcoat::cookie::time::Duration::hours(JWT_EXPIRATION_HOURS))
                .build();

            cookies(cx).add(cookie);

            if topcoat::htmx::hx_request(cx) {
                response_headers(cx).append(
                    http::header::HeaderName::from_static("hx-redirect"),
                    http::HeaderValue::from_static("/"),
                );
                return view! { cx => "" }.single().await;
            } else {
                return Err(see_other("/").into());
            }
        }
    }

    // Invalid credentials: swap inline alert
    view! { cx =>
        <div class="rounded-md bg-destructive/15 p-3 text-destructive border border-destructive/20 flex items-center gap-2 text-sm">
            icon_alert_triangle(class: "size-4 shrink-0")
            <span>"Invalid username or password. Please check your credentials."</span>
        </div>
    }
    .single()
    .await
}

#[route(POST "/logout")]
pub async fn logout_action(cx: &Cx) -> Result<topcoat::router::response::Response> {
    let cookie = Cookie::build((AUTH_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .same_site(topcoat::cookie::SameSite::Lax)
        .max_age(topcoat::cookie::time::Duration::seconds(0))
        .build();

    cookies(cx).add(cookie);

    if topcoat::htmx::hx_request(cx) {
        let mut res = topcoat::router::response::Response::new(topcoat::router::Body::empty());
        res.headers_mut().insert(
            http::header::HeaderName::from_static("hx-redirect"),
            http::HeaderValue::from_static("/login"),
        );
        Ok(res)
    } else {
        let mut res = topcoat::router::response::Response::new(topcoat::router::Body::empty());
        *res.status_mut() = http::StatusCode::SEE_OTHER;
        res.headers_mut().insert(
            http::header::LOCATION,
            http::HeaderValue::from_static("/login"),
        );
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use topcoat::{
        asset::{AssetBundle, RouterBuilderAssetExt},
        cookie::RouterBuilderCookieExt,
        router::{to_bytes, Body, Router},
    };

    #[tokio::test]
    async fn test_login_page_renders_dark_mode_and_scripts() {
        let bundle = AssetBundle::load()
            .or_else(|_| AssetBundle::load_dir("target/debug/assets"))
            .unwrap();

        let router = Router::builder()
            .cookies()
            .page(login_page)
            .assets(bundle)
            .build();

        // 1. Test with dark theme cookie
        let request_dark = http::Request::builder()
            .uri("/login")
            .header("cookie", "theme=dark")
            .body(Body::empty())
            .unwrap();
        let response_dark = router.handle(request_dark).await;
        assert_eq!(response_dark.status(), http::StatusCode::OK);
        let bytes_dark = to_bytes(response_dark.into_body(), usize::MAX).await.unwrap();
        let html_dark = String::from_utf8(bytes_dark.to_vec()).unwrap();

        // Must include dark class on html tag
        assert!(
            html_dark.contains("<html lang=\"en\" class=\"dark\">"),
            "Expected <html lang=\"en\" class=\"dark\"> in HTML, got:\n{}",
            html_dark
        );
        // Must include theme-init script
        assert!(
            html_dark.contains("theme-init"),
            "Expected theme-init script tag in HTML"
        );
        // Must include app script
        assert!(
            html_dark.contains("app-"),
            "Expected app.js script tag in HTML"
        );
        // Must include theme controller button
        assert!(
            html_dark.contains("data-controller=\"theme\""),
            "Expected data-controller=\"theme\" in HTML"
        );

        // 2. Test without dark theme cookie (light default)
        let request_light = http::Request::builder()
            .uri("/login")
            .body(Body::empty())
            .unwrap();
        let response_light = router.handle(request_light).await;
        assert_eq!(response_light.status(), http::StatusCode::OK);
        let bytes_light = to_bytes(response_light.into_body(), usize::MAX).await.unwrap();
        let html_light = String::from_utf8(bytes_light.to_vec()).unwrap();

        // Should have empty class on html tag (not dark)
        assert!(
            html_light.contains("<html lang=\"en\" class=\"\">"),
            "Expected <html lang=\"en\" class=\"\"> in HTML, got:\n{}",
            html_light
        );
        // Still must include theme-init script for client-side resolution
        assert!(
            html_light.contains("theme-init"),
            "Expected theme-init script tag in HTML"
        );
        assert!(
            html_light.contains("app-"),
            "Expected app.js script tag in HTML"
        );
        assert!(
            html_light.contains("data-controller=\"theme\""),
            "Expected data-controller=\"theme\" in HTML"
        );
    }
}
