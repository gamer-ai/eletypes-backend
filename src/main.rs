use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use eletypes_backend::config::cors::configure_cors;
use eletypes_backend::utils::database::{configure_routes, connect_to_mongodb, get_server_address};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let address = get_server_address();
    let mongodb_client = connect_to_mongodb().await;

    println!("Server is running on {}", address);

    HttpServer::new(move || {
        App::new()
            .wrap(configure_cors())
            .app_data(web::Data::new(mongodb_client.clone()))
            .configure(configure_routes)
    })
    .bind(address)?
    .run()
    .await
}
