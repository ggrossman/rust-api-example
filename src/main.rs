use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_service::Service;
use mongodb::{Client, options::ClientOptions};
use middleware::auth::Authenticator;

mod config;
mod models;
mod views;
mod controllers;
mod middleware;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Authenticator)
            .configure(controllers::user_controller::init)
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await
}
