use crate::constants::{COLL_NAME, DB_NAME};
use crate::repositories::user_repository::update_user_high_scores;
use crate::repositories::user_repository::{
    fetch_filtered_users, fetch_user_and_handle_response, update_user_and_handle_respond,
    LeaderboardEntry, LeaderboardResponse, ScoreUpdateRequest, TimerDurationQuery,
};
use actix_web::{web, HttpResponse};
use mongodb::bson;
use mongodb::Client;

pub async fn update_user_score(
    client: web::Data<Client>,
    username: web::Path<String>,
    form: web::Json<ScoreUpdateRequest>,
) -> HttpResponse {
    let collection = client
        .database(DB_NAME)
        .collection::<bson::Document>(COLL_NAME);
    let username_str = username.into_inner();
    let score_update = form.into_inner();

    // Fetch user from the database
    let mut user = match fetch_user_and_handle_response(&collection, &username_str).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    // Update user data
    update_user_high_scores(&mut user, score_update);

    // Update the user in the database
    update_user_and_handle_respond(&collection, &username_str, &user).await
}

pub async fn get_leaderboard_stats(
    client: web::Data<Client>,
    query: web::Query<TimerDurationQuery>,
) -> HttpResponse {
    let collection = client
        .database(DB_NAME)
        .collection::<bson::Document>(COLL_NAME);

    let timer_duration: &String = &query.timer_duration.to_string();

    // Fetch users with high scores for the specified timer duration
    let users = match fetch_filtered_users(&collection, timer_duration).await {
        Ok(users) => users,
        Err(_) => {
            return HttpResponse::InternalServerError().json(LeaderboardResponse {
                status: "error".to_string(),
                message: "Failed to fetch leaderboard data. Please try again later.".to_string(),
                leaderboard: vec![],
            });
        }
    };

    // Prepare leaderboard data
    let leaderboard: Vec<LeaderboardEntry> = users.into_iter().collect();

    HttpResponse::Ok().json(LeaderboardResponse {
        status: "success".to_string(),
        message: "Leaderboard stats retrieved successfully.".to_string(),
        leaderboard,
    })
}
