use actix_web::{test, App};
use mongodb::bson::{doc, Document};
use rust_users::models::user::{User, UserLogin};
use rust_users::controllers::user_controller::init;
use rust_users::utils::mongo::get_mongo_client;
use actix_web::http::header;

async fn get_auth_token() -> String {
    let mut app = test::init_service(App::new().configure(init)).await;

    let login_info = UserLogin {
        email: "test@example.com".to_string(),
        password: "secret".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_info)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    std::str::from_utf8(&body).unwrap().to_string()
}

#[actix_rt::test]
async fn test_login_success() {
    let mut app = test::init_service(App::new().configure(init)).await;

    let login_info = UserLogin {
        email: "test@example.com".to_string(),
        password: "secret".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_info)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_login_failure() {
    let mut app = test::init_service(App::new().configure(init)).await;

    let login_info = UserLogin {
        email: "test@example.com".to_string(),
        password: "wrong_password".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login_info)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_rt::test]
async fn test_create_user() {
    let token = get_auth_token().await;

    let mut app = test::init_service(App::new().configure(init)).await;

    let user_info = User {
        firstname: "John".to_string(),
        lastname: "Doe".to_string(),
        email: "john.doe@example.com".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&user_info)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_get_users() {
    let token = get_auth_token().await;

    let mut app = test::init_service(App::new().configure(init)).await;

    let req = test::TestRequest::get()
        .uri("/users")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(body_str.contains("\"data\":["));
}

#[actix_rt::test]
async fn test_delete_user() {
    let token = get_auth_token().await;

    let client = get_mongo_client().await.unwrap();
    let collection = client.database("rust-users").collection::<Document>("users");

    // Insert a test user to delete
    let user = doc! {
        "firstname": "Test",
        "lastname": "User",
        "email": "test.user@example.com",
    };
    let insert_result = collection.insert_one(user.clone(), None).await.unwrap();
    let user_id = insert_result.inserted_id.as_object_id().unwrap().to_hex();

    let mut app = test::init_service(App::new().configure(init)).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/users/{}", user_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}
