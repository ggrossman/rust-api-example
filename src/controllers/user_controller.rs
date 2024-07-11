use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/login")
            .route(web::post().to(crate::views::user_views::login))
    )
    .service(
        web::resource("/users")
            .route(web::get().to(crate::views::user_views::get_users))
            .route(web::post().to(crate::views::user_views::new_user))
    )
    .service(
        web::resource("/users/{id}")
            .route(web::delete().to(crate::views::user_views::delete_user))
    );
}
