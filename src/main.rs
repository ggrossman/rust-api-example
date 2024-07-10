use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, web, App, Error, HttpServer, HttpRequest, HttpResponse, Responder, middleware::Logger, body::EitherBody};
use futures_util::future::{ok, Ready};
use futures_util::TryStreamExt;
use mongodb::{Client, options::ClientOptions, bson::{doc, oid::ObjectId, Document}};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use std::pin::Pin;
use std::future::Future;
use std::boxed::Box;
use futures_util::TryFutureExt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct User {
    firstname: String,
    lastname: String,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserLogin {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionJWT {
    iat: u64,
    exp: u64,
    email: String,
    password: String,
}

static AUTH_SECRET: &str = "your_secret_key";
static JWT_EXPIRATION_SECS: u64 = 3600;

async fn get_data_string(result: mongodb::error::Result<Document>) -> Result<web::Json<Document>, String> {
    match result {
        Ok(doc) => Ok(web::Json(doc)),
        Err(e) => Err(format!("{}", e)),
    }
}

struct Authenticator;

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

struct AuthenticatorMiddleware<S> {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Authenticator)
            .service(
                web::resource("/login")
                    .route(web::post().to(login))
            )
            .service(
                web::resource("/users")
                    .route(web::get().to(get_users))
                    .route(web::post().to(new_user))
            )
            .service(
                web::resource("/users/{id}")
                    .route(web::delete().to(delete_user))
            )
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await
}

async fn login(info: web::Json<UserLogin>) -> impl Responder {
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

async fn get_users() -> impl Responder {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();

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

async fn new_user(info: web::Json<User>) -> impl Responder {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();

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

async fn delete_user(req: HttpRequest) -> impl Responder {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();

    let collection = client.database("rust-users").collection::<Document>("users");
    let object_id = req.match_info().get("id").unwrap();

    let id = ObjectId::parse_str(object_id).unwrap();

    match collection.delete_one(doc! {"_id": id}, None).await {
        Ok(_) => HttpResponse::Ok().body("Item deleted!"),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}
