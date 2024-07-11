use mongodb::{Client, options::ClientOptions};

pub async fn get_mongo_client() -> mongodb::error::Result<Client> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    Ok(client)
}
