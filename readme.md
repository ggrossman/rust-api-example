# Rust API Example

This repo shows how to implement a RESTful API in Rust with **[Actix Web](https://actix.rs/)** and the **[MongoDB Rust Driver](https://www.mongodb.com/docs/drivers/rust/current/)**.

This REST API uses simple JWT authentication. Note that it does not actually integrate with Auth0!

## Prerequisites

You will need to have MongoDB installed and running on `localhost:27017`. If on macOS, for instance, follow MongoDB's [installation instructions](https://www.mongodb.com/docs/manual/tutorial/install-mongodb-on-os-x/).

## Usage

Issue a POST to the `/login` route to obtain a JWT bearer token.

```
% curl -v http://127.0.0.1:9000/login -X POST -d '{"email": "Joe User", "password": "secret"}' -H "Content-Type: application/json"
Note: Unnecessary use of -X or --request, POST is already inferred.
*   Trying 127.0.0.1:9000...
* Connected to 127.0.0.1 (127.0.0.1) port 9000
> POST /login HTTP/1.1
> Host: 127.0.0.1:9000
> User-Agent: curl/8.4.0
> Accept: */*
> Content-Type: application/json
> Content-Length: 43
>
< HTTP/1.1 200 OK
< content-length: 180
< date: Wed, 10 Jul 2024 14:20:55 GMT
<
* Connection #0 to host 127.0.0.1 left intact
eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MjA2MjEyNTYsImV4cCI6MTcyMDYyNDg1NiwiZW1haWwiOiJKb2UgVXNlciIsInBhc3N3b3JkIjoic2VjcmV0In0.CLi9Jc34GUOMuHuK7KDN2BUI2-vX6KI4yfnIN6ngm0E
```

The JWT bearer token can now be specified to the protected routes such as `/users`.

```
% curl -v -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MjA2MjEyNTYsImV4cCI6MTcyMDYyNDg1NiwiZW1haWwiOiJKb2UgVXNlciIsInBhc3N3b3JkIjoic2VjcmV0In0.CLi9Jc34GUOMuHuK7KDN2BUI2-vX6KI4yfnIN6ngm0E" http://127.0.0.1:9000/users -X POST -d '{"firstname": "Joe", "lastname": "User", "email": "joe@example.org"}' -H "Content-Type: application/json" 
Note: Unnecessary use of -X or --request, POST is already inferred.
*   Trying 127.0.0.1:9000...
* Connected to 127.0.0.1 (127.0.0.1) port 9000
> POST /users HTTP/1.1
> Host: 127.0.0.1:9000
> User-Agent: curl/8.4.0
> Accept: */*
> Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MjA2MjEyNTYsImV4cCI6MTcyMDYyNDg1NiwiZW1haWwiOiJKb2UgVXNlciIsInBhc3N3b3JkIjoic2VjcmV0In0.CLi9Jc34GUOMuHuK7KDN2BUI2-vX6KI4yfnIN6ngm0E
> Content-Type: application/json
> Content-Length: 68
>
< HTTP/1.1 200 OK
< content-length: 11
< date: Wed, 10 Jul 2024 14:23:07 GMT
<
* Connection #0 to host 127.0.0.1 left intact
Item saved!

% curl -v -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MjA2MjEyNTYsImV4cCI6MTcyMDYyNDg1NiwiZW1haWwiOiJKb2UgVXNlciIsInBhc3N3b3JkIjoic2VjcmV0In0.CLi9Jc34GUOMuHuK7KDN2BUI2-vX6KI4yfnIN6ngm0E" http://127.0.0.1:9000/users
*   Trying 127.0.0.1:9000...
* Connected to 127.0.0.1 (127.0.0.1) port 9000
> GET /users HTTP/1.1
> Host: 127.0.0.1:9000
> User-Agent: curl/8.4.0
> Accept: */*
> Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MjA2MjEyNTYsImV4cCI6MTcyMDYyNDg1NiwiZW1haWwiOiJKb2UgVXNlciIsInBhc3N3b3JkIjoic2VjcmV0In0.CLi9Jc34GUOMuHuK7KDN2BUI2-vX6KI4yfnIN6ngm0E
>
< HTTP/1.1 200 OK
< content-length: 127
< content-type: application/json
< date: Wed, 10 Jul 2024 14:23:51 GMT
<
* Connection #0 to host 127.0.0.1 left intact
{"data":[{ "_id": ObjectId("668e994c7eba267f28496f8a"), "firstname": "Joe", "lastname": "User", "email": "joe@example.org" },]}
```

## Important Snippets

The simple example has `GET`, `POST`, and `DELETE` routes in the `main` function.

The **GET** `/users` route searches MongoDB for all users and then returns a JSON string of the data.

```rust
// src/main.rs

...

async fn get_data_string(result: mongodb::error::Result<Document>) -> Result<web::Json<Document>, String> {
    match result {
        Ok(doc) => Ok(web::Json(doc)),
        Err(e) => Err(format!("{}", e)),
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

...
```

The **POST** `/users` route takes JSON data and saves in the database. The data conforms to the `User` struct.

```rust
// src/main.rs

...

#[derive(Serialize, Deserialize)]
struct User {
    firstname: String,
    lastname: String,
    email: String,
}

...

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

...
```

The **DELETE** `/users/:user_id` takes an `objectid` as a parameter, decodes it into BSON, and deletes it from the database.

```rust
// src/main.rs

...

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

...
```

## What is Auth0?

Auth0 helps you to:

* Add authentication with [multiple authentication sources](https://docs.auth0.com/identityproviders), either social like **Google, Facebook, Microsoft Account, LinkedIn, GitHub, Twitter, Box, Salesforce, amont others**, or enterprise identity systems like **Windows Azure AD, Google Apps, Active Directory, ADFS or any SAML Identity Provider**.
* Add authentication through more traditional **[username/password databases](https://docs.auth0.com/mysql-connection-tutorial)**.
* Add support for **[linking different user accounts](https://docs.auth0.com/link-accounts)** with the same user.
* Support for generating signed [Json Web Tokens](https://docs.auth0.com/jwt) to call your APIs and **flow the user identity** securely.
* Analytics of how, when and where users are logging in.
* Pull data from other sources and add it to the user profile, through [JavaScript rules](https://docs.auth0.com/rules).

## Create a free account in Auth0

1. Go to [Auth0](https://auth0.com) and click Sign Up.
2. Use Google, GitHub or Microsoft Account to login.

## Issue Reporting

If you have found a bug or if you have a feature request, please report them at this repository issues section. Please do not report security vulnerabilities on the public GitHub issue tracker. The [Responsible Disclosure Program](https://auth0.com/whitehat) details the procedure for disclosing security issues.

## Author

[Auth0](auth0.com)

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file for more info.
