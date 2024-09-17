use crate::constants::{COLL_NAME, DB_NAME};
use crate::services::leaderboard_service::{fetch_filtered_users, get_total_document_count};
use crate::structs::leaderboard::{
    GetLeaderboardStatsQueries, LeaderboardEntry, LeaderboardResponse,
};

use actix_web::{web, HttpResponse};
use mongodb::bson;
use mongodb::Client;

pub async fn get_leaderboard_stats(
    client: web::Data<Client>,
    query: web::Query<GetLeaderboardStatsQueries>,
) -> HttpResponse {
    let collection = client
        .database(DB_NAME)
        .collection::<bson::Document>(COLL_NAME);

    let total_count = match get_total_document_count(&collection).await {
        Ok(count) => count,
        Err(response) => return response,
    };

    let timer_duration = query.timer_duration.to_string();
    let language = query.language.to_string();
    let difficulty = query.difficulty.to_string();

    let users = fetch_filtered_users(
        &collection,
        &timer_duration,
        &query.page,
        &query.limit,
        &language,
        &difficulty,
    )
    .await;

    if users.is_err() {
        return HttpResponse::InternalServerError().json(LeaderboardResponse {
            status: "error".to_string(),
            message: "Failed to fetch leaderboard data. Please try again later.".to_string(),
            leaderboard: vec![],
            total_count: 0,
        });
    }

    let leaderboard: Vec<LeaderboardEntry> = users.unwrap().into_iter().collect();

    HttpResponse::Ok().json(LeaderboardResponse {
        status: "success".to_string(),
        message: "Leaderboard stats retrieved successfully.".to_string(),
        leaderboard,
        total_count,
    })
}
