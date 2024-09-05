use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use eletypes_backend::routes::score_routes::update_user_score;
use eletypes_backend::routes::user_routes::{add_user, get_user};
use eletypes_backend::utils::database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Configure CORS
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("localhost:{}", port);
    let client = database::connect_to_mongodb().await;

    println!("Server is running on {}", address);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // Allow this origin
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"]) // Allow these HTTP methods
            .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE]) // Allow these headers
            .max_age(3600); // Cache preflight responses for 1 hour

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(client.clone()))
            .route("/get_user/{username}", web::get().to(get_user))
            .route("/add_user", web::post().to(add_user))
            .route(
                "/update_user_score/{username}",
                web::post().to(update_user_score),
            )
    })
    .bind(address)?
    .run()
    .await
}
