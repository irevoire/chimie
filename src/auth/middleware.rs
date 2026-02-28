use std::{
    future::{Ready, ready},
    rc::Rc,
    str::FromStr,
};

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    web::Data,
};
use futures_util::future::LocalBoxFuture;

use crate::auth::{error::AuthenticationError, token_db::AccessTokenDatabase};

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "snake_case", deny_unknown_fields)]
#[repr(u8)]
enum AuthType {
    Password,
}

impl FromStr for AuthType {
    type Err = AuthenticationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "password" => Ok(AuthType::Password),
            s => Err(AuthenticationError::WrongAuthTypeValue(s.to_string())),
        }
    }
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "snake_case", deny_unknown_fields)]
#[repr(u8)]
enum IsAuthenticated {
    True,
}

impl FromStr for IsAuthenticated {
    type Err = AuthenticationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "true" => Ok(IsAuthenticated::True),
            s => Err(AuthenticationError::WrongIsAuthenticatedValue(
                s.to_string(),
            )),
        }
    }
}

#[derive(Debug, Default, facet::Facet)]
#[facet(rename_all = "snake_case", deny_unknown_fields)]
struct Cookie {
    immich_access_token: Option<String>,
    immich_auth_type: Option<AuthType>,
    immich_is_authenticated: Option<IsAuthenticated>,
}

const ACCESS_TOKEN: &str = "immich_access_token";
const AUTH_TYPE: &str = "immich_auth_type";
const IS_AUTHENTICATED: &str = "immich_is_authenticated";

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
#[derive(Clone)]
pub struct Auth(pub Data<AccessTokenDatabase>);

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            db: self.0.clone(),
        }))
    }
}

pub struct AuthMiddleware<S> {
    db: Data<AccessTokenDatabase>,
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let db = self.db.clone();

        Box::pin(async move {
            let Some(cookie) = req.headers().get("Cookie") else {
                return Result::<_, Self::Error>::Err(
                    AuthenticationError::MissingAuthCookie.into(),
                );
            };
            let str_cookie = cookie
                .to_str()
                .map_err(AuthenticationError::NonUtf8Cookie)?;
            let mut cookie = Cookie::default();

            for (idx, entry) in str_cookie
                .split(';')
                .map(|s| s.trim().split_once('='))
                .enumerate()
            {
                let (field, value) =
                    entry.ok_or_else(|| AuthenticationError::MalformedCookie(idx))?;

                if field == ACCESS_TOKEN {
                    if cookie.immich_access_token.is_some() {
                        return Err(AuthenticationError::DuplicateField(ACCESS_TOKEN).into());
                    } else {
                        cookie.immich_access_token = Some(value.to_string());
                    }
                } else if field == AUTH_TYPE {
                    if cookie.immich_auth_type.is_some() {
                        return Err(AuthenticationError::DuplicateField(AUTH_TYPE).into());
                    } else {
                        cookie.immich_auth_type = Some(AuthType::from_str(value)?);
                    }
                } else if field == IS_AUTHENTICATED {
                    if cookie.immich_is_authenticated.is_some() {
                        return Err(AuthenticationError::DuplicateField(IS_AUTHENTICATED).into());
                    } else {
                        match IsAuthenticated::from_str(value) {
                            Ok(authenticated) => {
                                cookie.immich_is_authenticated = Some(authenticated)
                            }
                            Err(err) => return Err(err.into()),
                        };
                    }
                } else {
                    let field = field.to_string();
                    return Err(AuthenticationError::UnexpectedField(field).into());
                }
            }
            if cookie.immich_access_token.is_none() {
                return Err(AuthenticationError::MissingField(ACCESS_TOKEN).into());
            }
            if cookie.immich_auth_type.is_none() {
                return Err(AuthenticationError::MissingField(AUTH_TYPE).into());
            }
            if cookie.immich_is_authenticated.is_none() {
                return Err(AuthenticationError::MissingField(IS_AUTHENTICATED).into());
            }
            let uuid = cookie.immich_access_token.unwrap();
            if db.get(uuid).await.is_some() {
                service.call(req).await
            } else {
                Err(AuthenticationError::UnknownAccessToken.into())
            }
        })
    }
}
