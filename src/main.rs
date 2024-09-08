use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use eletypes_backend::config::cors::configure_cors;
use eletypes_backend::config::database::{connect_to_mongodb, get_server_address};
use eletypes_backend::routes::{
    leaderboard_routes::configure_leaderboard_routes, user_routes::configure_user_routes,
};

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
            .configure(configure_leaderboard_routes)
            .configure(configure_user_routes)
    })
    .bind(address)?
    .run()
    .await
}
