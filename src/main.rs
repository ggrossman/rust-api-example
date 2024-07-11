use actix_web::{App, HttpServer, middleware::Logger};
use rust_users::middleware::auth::Authenticator;
use rust_users::controllers;

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
