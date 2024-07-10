use actix_web::{HttpRequest, HttpResponse, Responder, web};
use mongodb::bson::{doc, Document, oid::ObjectId};
use crate::models::user::{User, UserLogin};
use crate::utils::mongo::get_mongo_client;
use futures_util::TryStreamExt;
use jsonwebtoken::{encode, Header, EncodingKey};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::{AUTH_SECRET, JWT_EXPIRATION_SECS};
use crate::models::user::SessionJWT;

pub async fn login(info: web::Json<UserLogin>) -> impl Responder {
    let email = info.email.clone();
    let password = info.password.clone();

    if password == "secret" {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let claims = SessionJWT { iat: since_the_epoch, exp: since_the_epoch+JWT_EXPIRATION_SECS, email, password };
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(AUTH_SECRET.as_ref())).unwrap();

        HttpResponse::Ok().body(token)
    } else {
        HttpResponse::BadRequest().body("Incorrect username or password")
    }
}

pub async fn get_users() -> impl Responder {
    let client = get_mongo_client().await.unwrap();
    let collection = client.database("rust-users").collection::<Document>("users");
    let mut cursor = collection.find(None, None).await.unwrap();

    let mut data_result = "{\"data\":[".to_owned();

    while let Some(result) = cursor.try_next().await.unwrap() {
        match get_data_string(Ok(result)).await {
            Ok(data) => {
                let string_data = format!("{},", data.into_inner());
                data_result.push_str(&string_data);
            }
            Err(e) => return HttpResponse::InternalServerError().body(e),
        }
    }

    data_result.push_str("]}");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(data_result)
}

pub async fn new_user(info: web::Json<User>) -> impl Responder {
    let client = get_mongo_client().await.unwrap();
    let collection = client.database("rust-users").collection::<Document>("users");
    let user = doc! {
        "firstname": &info.firstname,
        "lastname": &info.lastname,
        "email": &info.email,
    };

    match collection.insert_one(user, None).await {
        Ok(_) => HttpResponse::Ok().body("Item saved!"),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

pub async fn delete_user(req: HttpRequest) -> impl Responder {
    let client = get_mongo_client().await.unwrap();
    let collection = client.database("rust-users").collection::<Document>("users");
    let object_id = req.match_info().get("id").unwrap();

    let id = ObjectId::parse_str(object_id).unwrap();

    match collection.delete_one(doc! {"_id": id}, None).await {
        Ok(_) => HttpResponse::Ok().body("Item deleted!"),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

async fn get_data_string(result: mongodb::error::Result<Document>) -> Result<web::Json<Document>, String> {
    match result {
        Ok(doc) => Ok(web::Json(doc)),
        Err(e) => Err(format!("{}", e)),
    }
}
