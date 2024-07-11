use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse, body::EitherBody};
use futures_util::future::{ok, Ready};
use futures_util::TryFutureExt;
use jsonwebtoken::{decode, Validation, DecodingKey};
use std::pin::Pin;
use std::future::Future;
use std::boxed::Box;
use crate::models::user::SessionJWT;
use crate::config::AUTH_SECRET;

pub struct Authenticator;

impl<S, B> Transform<S, ServiceRequest> for Authenticator
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthenticatorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticatorMiddleware { service })
    }
}

pub struct AuthenticatorMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticatorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header.to_str().unwrap_or(""),
            None => "",
        };

        let jwt = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            ""
        };

        if req.method() == "OPTIONS" || req.path() == "/login" {
            return Box::pin(self.service.call(req).map_ok(|res| res.map_into_left_body()));
        }

        let token_data = decode::<SessionJWT>(&jwt, &DecodingKey::from_secret(AUTH_SECRET.as_ref()), &Validation::default());

        if let Ok(_) = token_data {
            Box::pin(self.service.call(req).map_ok(|res| res.map_into_left_body()))
        } else {
            let res = req.into_response(HttpResponse::Forbidden().finish().map_into_right_body());
            Box::pin(async { Ok(res) })
        }
    }
}
