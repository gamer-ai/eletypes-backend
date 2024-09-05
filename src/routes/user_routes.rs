use crate::constants::{COLL_NAME, DB_NAME};
use crate::models::user_model::User;
use actix_web::{web, HttpResponse};
use mongodb::{bson::doc, Client, Collection};

// Adds a new user to the "users" collection in the database.
pub async fn add_user(client: web::Data<Client>, form: web::Json<User>) -> HttpResponse {
    println!("{:?}", form);
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(form.into_inner()).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// Gets the user with the supplied username.
pub async fn get_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    match collection.find_one(doc! { "username": &username }).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
