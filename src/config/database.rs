use dotenv::dotenv;
use mongodb::Client;

pub async fn connect_to_mongodb() -> Client {
    dotenv().ok();

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    match Client::with_uri_str(uri).await {
        Ok(client) => client,
        Err(e) => {
            panic!("Failed to connect to MongoDB: {:?}", e);
        }
    }
}

pub fn get_server_address() -> String {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    format!("localhost:{}", port)
}
