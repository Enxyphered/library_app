use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use http::{header::COOKIE, StatusCode};
use topcoat::{
    context::Cx,
    htmx::hx_request,
    router::{
        request::{client_ip, headers, method, remote_addr, uri},
        response::IntoResponse,
        Body, Layer, LayerFuture, Next, Path, RouterBuilder,
    },
};

use crate::auth::jwt::{verify_jwt, AUTH_COOKIE_NAME};

/// Global latency telemetry counters for tracking percentiles & sub-1ms compliance.
#[derive(Debug, Default)]
pub struct LatencyTelemetry {
    pub total_requests: AtomicU64,
    pub sub_1ms_requests: AtomicU64,
    pub sub_500us_requests: AtomicU64,
}

pub static LATENCY_TELEMETRY: LatencyTelemetry = LatencyTelemetry {
    total_requests: AtomicU64::new(0),
    sub_1ms_requests: AtomicU64::new(0),
    sub_500us_requests: AtomicU64::new(0),
};

/// Helper to extract real client IP address, checking reverse proxy headers (X-Forwarded-For, X-Real-IP) first.
fn extract_client_ip(cx: &Cx) -> String {
    let hdrs = headers(cx);

    // 1. Check X-Forwarded-For: Caddy appends or sets client IP here (format: client, proxy1, proxy2)
    if let Some(xff) = hdrs.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(client_ip) = xff.split(',').next().map(str::trim).filter(|s| !s.is_empty()) {
            return client_ip.to_string();
        }
    }

    // 2. Check X-Real-IP
    if let Some(x_real_ip) = hdrs.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        let trimmed = x_real_ip.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // 3. Check Topcoat's resolved client_ip or remote_addr
    client_ip(cx)
        .map(|ip| ip.to_string())
        .or_else(|| remote_addr(cx).map(|addr| addr.ip().to_string()))
        .unwrap_or_else(|| "-".to_string())
}

/// Helper to safely inspect the auth cookie from headers without panicking if CookieJarCell is absent.
fn extract_auth_username(cx: &Cx) -> Option<String> {
    let cookie_header = headers(cx).get(COOKIE)?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        if let (Some(name), Some(val)) = (parts.next(), parts.next()) {
            if name == AUTH_COOKIE_NAME {
                if let Ok(user) = verify_jwt(val) {
                    return Some(user.username);
                }
            }
        }
    }
    None
}

/// Helper to format HTTP status code with ANSI color codes.
fn format_status_code(status: StatusCode) -> String {
    let status_code = status.as_u16();
    let (status_color, reset_color) = if status_code >= 500 {
        ("\x1b[31;1m", "\x1b[0m") // Bright Red
    } else if status_code >= 400 {
        ("\x1b[33;1m", "\x1b[0m") // Bright Yellow
    } else if status_code >= 300 {
        ("\x1b[36;1m", "\x1b[0m") // Bright Cyan
    } else {
        ("\x1b[32;1m", "\x1b[0m") // Bright Green
    };
    format!("{}{}{}", status_color, status_code, reset_color)
}

/// A topcoat layer that logs incoming HTTP requests and responses with:
/// - Method and URI
/// - Response HTTP status code
/// - Total elapsed time spent per request
/// - Client IP address
/// - HTMX request indicator & target (if applicable)
/// - Authenticated username (if logged in)
#[derive(Debug, Clone, Copy, Default)]
pub struct RequestLoggingLayer;

impl RequestLoggingLayer {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Layer for RequestLoggingLayer {
    fn path(&self) -> Option<&Path> {
        // None wraps every request (including unmatched paths/404s)
        None
    }

    fn handle<'a>(&'a self, cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
        Box::pin(async move {
            let start = Instant::now();

            // Extract request details before running handler
            let req_method = method(cx).clone();
            let req_uri = uri(cx).to_string();
            let is_htmx = hx_request(cx);
            let hx_target = headers(cx)
                .get("hx-target")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            let user = extract_auth_username(cx);
            let ip = extract_client_ip(cx);

            let result = next.run(cx, body).await;
            let elapsed = start.elapsed();

            // Record latency telemetry
            let _total = LATENCY_TELEMETRY.total_requests.fetch_add(1, Ordering::Relaxed) + 1;
            let is_sub_1ms = elapsed.as_micros() < 1000;
            if is_sub_1ms {
                LATENCY_TELEMETRY.sub_1ms_requests.fetch_add(1, Ordering::Relaxed);
            }
            if elapsed.as_micros() < 500 {
                LATENCY_TELEMETRY.sub_500us_requests.fetch_add(1, Ordering::Relaxed);
            }

            // Format elapsed duration cleanly
            let elapsed_str = if elapsed.as_millis() >= 1 {
                format!("{:.2}ms", elapsed.as_secs_f64() * 1000.0)
            } else {
                format!("{}µs", elapsed.as_micros())
            };

            // Determine status code
            let (status, is_err) = match &result {
                Ok(response) => (response.status(), false),
                Err(error) => {
                    // Turn error into response representation to obtain its HTTP status code
                    let error_status = error
                        .clone()
                        .into_response(cx)
                        .map(|resp| resp.status())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    (error_status, true)
                }
            };

            // Colorize status code with ANSI escape codes
            let status_formatted = format_status_code(status);

            let mut extras = Vec::new();
            if is_htmx {
                if let Some(target) = hx_target {
                    extras.push(format!("HTMX:#{}", target));
                } else {
                    extras.push("HTMX".to_string());
                }
            }
            if let Some(username) = user {
                extras.push(format!("user:{}", username));
            }
            extras.push(format!("ip:{}", ip));

            let extras_str = if extras.is_empty() {
                String::new()
            } else {
                format!(" [{}]", extras.join(" | "))
            };

            let err_marker = if is_err { " [err]" } else { "" };

            println!(
                "\x1b[90m[HTTP]\x1b[0m {} {} -> {} in \x1b[1m{}\x1b[0m{}{}",
                req_method,
                req_uri,
                status_formatted,
                elapsed_str,
                extras_str,
                err_marker
            );

            result
        })
    }
}

/// Helper extension trait on `RouterBuilder` to register logging conveniently.
pub trait RouterBuilderLoggingExt {
    #[must_use]
    fn logging(self) -> Self;
}

impl RouterBuilderLoggingExt for RouterBuilder {
    fn logging(self) -> Self {
        self.layer(RequestLoggingLayer::new())
    }
}

