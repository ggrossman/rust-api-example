use mongodb::{Collection, error::Result, bson::{doc, Document, oid::ObjectId}};
use serde::{Deserialize, Serialize};
use crate::utils::mongo::get_mongo_client;
use futures_util::TryStreamExt;
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum UserError {
    #[error("MongoDB error: {0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("Invalid ObjectId: {0}")]
    InvalidObjectId(#[from] mongodb::bson::oid::Error),
}

impl User {
    pub async fn get_collection() -> Collection<Document> {
        let client = get_mongo_client().await.unwrap();
        client.database("rust-users").collection("users")
    }

    pub async fn find_all() -> Result<Vec<Document>> {
        let collection = Self::get_collection().await;
        let mut cursor = collection.find(None, None).await?;
        let mut users = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            users.push(doc);
        }
        Ok(users)
    }

    pub async fn insert(user: User) -> Result<()> {
        let collection = Self::get_collection().await;
        let doc = doc! {
            "firstname": user.firstname,
            "lastname": user.lastname,
            "email": user.email,
        };
        collection.insert_one(doc, None).await?;
        Ok(())
    }

    pub async fn delete_by_id(id: &str) -> std::result::Result<(), UserError> {
        let collection = Self::get_collection().await;
        let object_id = ObjectId::parse_str(id)?;
        collection.delete_one(doc! { "_id": object_id }, None).await?;
        Ok(())
    }
}
