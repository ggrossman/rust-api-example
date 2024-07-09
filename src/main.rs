use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Responder, middleware::Logger};
use mongodb::{Client, options::ClientOptions, bson::{doc, oid::ObjectId, Document}};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use futures::StreamExt;

#[derive(Serialize, Deserialize)]
struct User {
    firstname: String,
    lastname: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct UserLogin {
    email: String,
    password: String,
}

static AUTH_SECRET: &str = "your_secret_key";

async fn get_data_string(result: mongodb::error::Result<Document>) -> Result<web::Json<Document>, String> {
    match result {
        Ok(doc) => Ok(web::Json(doc)),
        Err(e) => Err(format!("{}", e)),
    }
}

async fn authenticator(
    req: HttpRequest,
    srv: &dyn actix_service::Service<
        HttpRequest,
        Response = HttpResponse,
        Error = actix_web::Error,
        Future = impl std::future::Future<Output = Result<HttpResponse, actix_web::Error>>,
    >,
) -> impl Responder {
    if req.method() == "OPTIONS" {
        return srv.call(req).await;
    }

    if req.path() == "/login" {
        return srv.call(req).await;
    }

    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or(""),
        None => "",
    };

    let jwt = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        ""
    };

    let token_data = decode::<UserLogin>(&jwt, &DecodingKey::from_secret(AUTH_SECRET.as_ref()), &Validation::default());

    match token_data {
        Ok(_) => srv.call(req).await,
        Err(_) => Ok(HttpResponse::Forbidden().finish()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
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
        let claims = UserLogin { email, password };
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

    while let Some(result) = cursor.next().await {
        match get_data_string(result).await {
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
