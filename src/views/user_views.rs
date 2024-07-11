use actix_web::{HttpRequest, HttpResponse, Responder, web};
use crate::models::user::{User, UserLogin, SessionJWT, UserError};
use jsonwebtoken::{encode, Header, EncodingKey};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::{AUTH_SECRET, JWT_EXPIRATION_SECS};
use serde_json::json;

pub async fn login(info: web::Json<UserLogin>) -> impl Responder {
    let email = info.email.clone();
    let password = info.password.clone();

    if password == "secret" {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let claims = SessionJWT { iat: since_the_epoch, exp: since_the_epoch + JWT_EXPIRATION_SECS, email, password };
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(AUTH_SECRET.as_ref())).unwrap();

        HttpResponse::Ok().body(token)
    } else {
        HttpResponse::BadRequest().body("Incorrect username or password")
    }
}

pub async fn get_users() -> impl Responder {
    match User::find_all().await {
        Ok(users) => {
            let data_result = json!({ "data": users });
            HttpResponse::Ok()
                .content_type("application/json")
                .json(data_result)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

pub async fn new_user(info: web::Json<User>) -> impl Responder {
    match User::insert(info.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("Item saved!"),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

pub async fn delete_user(req: HttpRequest) -> impl Responder {
    let object_id = req.match_info().get("id").unwrap();
    match User::delete_by_id(object_id).await {
        Ok(_) => HttpResponse::Ok().body("Item deleted!"),
        Err(UserError::Mongo(e)) => HttpResponse::InternalServerError().body(format!("{}", e)),
        Err(UserError::InvalidObjectId(e)) => HttpResponse::BadRequest().body(format!("Invalid ObjectId: {}", e)),
    }
}
