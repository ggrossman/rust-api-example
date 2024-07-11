use mongodb::{Client, options::ClientOptions, error::Result};

pub async fn get_mongo_client() -> Result<Client> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    Ok(client)
}
