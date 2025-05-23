// web/mw_auth.rs
use crate::ctx::Ctx;
use crate::model::ModelController;
use crate::web::AUTH_TOKEN;
use crate::{Error, Result};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};
use time::Duration;

pub async fn mw_require_auth(
    ctx: Result<Ctx>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");
    ctx?;

    Ok(next.run(req).await)
}

// web/mw_auth.rs - Updated cookie removal logic
pub async fn mw_ctx_resolver(
    _mc: State<ModelController>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    // Compute Result<Ctx>.
    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, username, _exp, _sign)) => {
            // TODO: Token components validations.
            Ok(Ctx::new(user_id, username))
        }
        Err(e) => Err(e),
    };

    // Remove the cookie if something went wrong other than NoAuthTokenCookie.
    if result_ctx.is_err()
        && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie))
    {
        // Create a cookie with the same name but empty value, path and with negative max_age
        let mut cookie = Cookie::new(AUTH_TOKEN, "");
        cookie.set_path("/");
        cookie.set_max_age(Duration::seconds(-1));
        
        
        // Add/replace the cookie (effectively removing it)
        cookies.add(cookie);
        
        println!("->> {:<12} - mw_ctx_resolver - Invalid token detected, cookie removed", "MIDDLEWARE");
    }

    // Store the ctx_result in the request extension.
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

// region: --- Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()
    }
}
// endregion: --- Ctx Extractor

/// Parse a token of format minecraft-[user-id].[username].[expiration].[signature]
/// Returns (user_id, username, expiration, signature)
fn parse_token(token: String) -> Result<(String, String, String, String)> {
    let (_whole, user_id, username, exp, sign) = regex_captures!(
        r#"^minecraft-(.+)\.(.+)\.(.+)\.(.+)"#, // a literal regex
        &token
    )
    .ok_or(Error::AuthFailTokenWrongFormat)?;

    Ok((user_id.to_string(), username.to_string(), exp.to_string(), sign.to_string()))
}