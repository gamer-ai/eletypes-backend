use dotenv::dotenv;
use mongodb::Client;

pub async fn connect_to_mongodb() -> Client {
    dotenv().ok();

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");

    client
}
