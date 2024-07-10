use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionJWT {
    pub iat: u64,
    pub exp: u64,
    pub email: String,
    pub password: String,
}
