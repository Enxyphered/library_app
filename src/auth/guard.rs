use http::{header::HeaderName, HeaderValue};
use topcoat::{
    context::Cx,
    cookie::{cookies, Cookies},
    htmx::hx_request,
    router::{
        error::see_other,
        response::response_headers,
    },
    Result,
};

use crate::auth::jwt::{verify_jwt, AuthenticatedUser, AUTH_COOKIE_NAME};

/// Attempts to extract the authenticated user from the request's `auth_token` cookie.
pub fn current_user(cx: &Cx) -> Option<AuthenticatedUser> {
    let jar = cookies(cx);
    let cookie = jar.get(AUTH_COOKIE_NAME)?;
    verify_jwt(cookie.value()).ok()
}

/// Enforces authentication.
/// If not authenticated:
/// - HTMX requests receive `HX-Redirect: /login` along with standard redirect.
/// - Standard browser requests receive HTTP 303 See Other `Location: /login`.
pub fn require_auth(cx: &Cx) -> Result<AuthenticatedUser> {
    if let Some(user) = current_user(cx) {
        Ok(user)
    } else {
        if hx_request(cx) {
            response_headers(cx).append(
                HeaderName::from_static("hx-redirect"),
                HeaderValue::from_static("/login"),
            );
        }
        Err(see_other("/login").into())
    }
}
