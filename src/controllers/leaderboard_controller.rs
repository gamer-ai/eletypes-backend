use crate::constants::{COLL_NAME, DB_NAME};
use crate::models::leaderboard::{LeaderboardEntry, LeaderboardResponse, TimerDurationQuery};
use crate::services::leaderboard_service::fetch_filtered_users;

use actix_web::{web, HttpResponse};
use mongodb::bson;
use mongodb::Client;

pub async fn get_leaderboard_stats(
    client: web::Data<Client>,
    query: web::Query<TimerDurationQuery>,
) -> HttpResponse {
    let collection = client
        .database(DB_NAME)
        .collection::<bson::Document>(COLL_NAME);

    let timer_duration: &str = &query.timer_duration;
    let page: &str = &query.page;
    let limit: &str = &query.limit;

    let users = match fetch_filtered_users(&collection, timer_duration, page, limit).await {
        Ok(users) => users,
        Err(_) => {
            return HttpResponse::InternalServerError().json(LeaderboardResponse {
                status: "error".to_string(),
                message: "Failed to fetch leaderboard data. Please try again later.".to_string(),
                leaderboard: vec![],
            });
        }
    };

    let leaderboard: Vec<LeaderboardEntry> = users.into_iter().collect();

    HttpResponse::Ok().json(LeaderboardResponse {
        status: "success".to_string(),
        message: "Leaderboard stats retrieved successfully.".to_string(),
        leaderboard,
    })
}
