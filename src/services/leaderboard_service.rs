use crate::models::user::HighScores;
pub use crate::structs::leaderboard::{
    GetLeaderboardStatsRequest, LeaderboardEntry, LeaderboardResponse,
};
use actix_web::HttpResponse;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, from_bson, Bson, Document};
use mongodb::Collection;
use std::collections::HashMap;

pub async fn get_total_document_count(
    collection: &Collection<Document>,
) -> Result<i64, HttpResponse> {
    match collection.count_documents(doc! {}).await {
        Ok(count) => Ok(count as i64),
        Err(_) => Err(
            HttpResponse::InternalServerError().json(LeaderboardResponse {
                status: "error".to_string(),
                message: "Failed to fetch the total document count.".to_string(),
                leaderboard: vec![],
                total_count: 0,
            }),
        ),
    }
}

pub fn extract_leaderboard_entry(
    doc: &Document,
) -> Result<LeaderboardEntry, mongodb::error::Error> {
    let _id = doc
        .get_object_id("_id")
        .map(|id| id.to_string())
        .unwrap_or_else(|_| "".to_string());
    let username = doc
        .get_str("username")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "".to_string());
    let completed_tests = doc
        .get_i32("completed_tests")
        .map(|n| n as u32)
        .unwrap_or_default();

    let high_scores = extract_high_scores(doc)?;

    Ok(LeaderboardEntry {
        _id,
        username,
        completed_tests,
        high_scores,
    })
}

fn extract_high_scores(doc: &Document) -> Result<HighScores, mongodb::error::Error> {
    match doc.get("high_scores") {
        Some(Bson::Document(doc)) => {
            let high_scores: HighScores = from_bson(Bson::Document(doc.clone()))?;
            Ok(high_scores)
        }
        _ => Ok(HighScores {
            languages: HashMap::new(),
        }),
    }
}

pub async fn fetch_filtered_users(
    collection: &mongodb::Collection<Document>,
    timer_duration: &str,
    page: &str,
    limit: &str,
    language: &str,
    difficulty: &str,
) -> Result<Vec<LeaderboardEntry>, mongodb::error::Error> {
    let pipeline = create_aggregation_pipeline(timer_duration, page, limit, language, difficulty);

    let mut cursor = collection.aggregate(pipeline).await?;
    let mut users = Vec::new();

    while let Some(doc) = cursor.try_next().await? {
        match extract_leaderboard_entry(&doc) {
            Ok(entry) => users.push(entry),
            Err(e) => eprintln!("Error processing document: {:?}", e),
        }
    }

    // log_leaderboard_stats(&users);

    Ok(users)
}

pub fn create_aggregation_pipeline(
    timer_duration: &str,
    page: &str,
    limit: &str,
    language: &str,
    difficulty: &str,
) -> Vec<Document> {
    let page_number: usize = page.parse().unwrap_or(1);
    let limit_number: usize = limit.parse().unwrap_or(10);
    let skip_number = (page_number - 1) * limit_number;

    vec![
        doc! { "$match": {
            format!("high_scores.languages.{}.difficulties.{}.scores.{}", language, difficulty, timer_duration): { "$exists": true }
        }},
        doc! { "$project": {
            "_id": 1,
            "username": 1,
            "completed_tests": 1,
            format!("high_scores.languages.{}.difficulties.{}.scores.{}", language, difficulty, timer_duration): 1
        }},
        doc! { "$sort": {
            format!("high_scores.languages.{}.difficulties.{}.scores.{}.wpm", language, difficulty, timer_duration): -1
        }},
        doc! { "$skip": skip_number as i64 },
        doc! { "$limit": limit_number as i64 },
    ]
}

// fn log_leaderboard_stats(users: &[LeaderboardEntry]) {
//     println!(
//         "Leaderboard Stats: {}",
//         serde_json::to_string_pretty(&json!(users)).unwrap()
//     );
// }
