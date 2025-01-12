use axum::response::Response;

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;

use crate::helper::validate_token;
use axum_casbin::CasbinVals;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::typed_header::TypedHeaderRejection;
use axum_extra::TypedHeader;
use pasetors::claims::Claims;

pub async fn authentication_middleware(
    bearer: Result<TypedHeader<Authorization<Bearer>>, TypedHeaderRejection>,
    mut req: Request,
    next: Next,
) -> Response<Body> {
    let default_anon = CasbinVals {
        subject: String::from("anon"),
        domain: None,
    };
    let mut claims: Option<Claims> = None;
    match bearer {
        Ok(TypedHeader(Authorization(bearer))) => {
            if let Some((role, c)) = validate_token(bearer.token()) {
                let vals = CasbinVals {
                    subject: role,
                    domain: None,
                };
                claims = Some(c);
                req.extensions_mut().insert(vals);
            } else {
                req.extensions_mut().insert(default_anon);
            }
        }
        Err(_) => {
            req.extensions_mut().insert(default_anon);
        }
    }
    req.extensions_mut().insert(claims);
    next.run(req).await
}
