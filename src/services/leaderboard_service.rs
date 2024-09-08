pub use crate::models::leaderboard::{
    GetLeaderboardStatsRequest, LeaderboardEntry, LeaderboardResponse,
};
use crate::models::user::ScoreEntry;
use futures_util::TryStreamExt;
use mongodb::bson::Document;
use mongodb::bson::{doc, from_bson, Bson};
// use serde_json::json;
use std::collections::HashMap;

pub fn extract_leaderboard_entry(
    doc: &Document,
) -> Result<LeaderboardEntry, mongodb::error::Error> {
    let _id = doc.get_object_id("_id").unwrap_or_default().to_string();
    let username = doc.get_str("username").unwrap_or_default().to_string();
    let completed_tests = doc.get_i32("completed_tests").unwrap_or_default() as u32;

    let high_scores = extract_high_scores(doc)?;

    Ok(LeaderboardEntry {
        _id,
        username,
        completed_tests,
        high_scores,
    })
}

fn extract_high_scores(
    doc: &Document,
) -> Result<HashMap<String, ScoreEntry>, mongodb::error::Error> {
    match doc.get("high_scores") {
        Some(Bson::Document(doc)) => {
            let score_entry: HashMap<String, ScoreEntry> = from_bson(Bson::Document(doc.clone()))?;
            Ok(score_entry)
        }
        _ => Ok(HashMap::new()),
    }
}

pub async fn fetch_filtered_users(
    collection: &mongodb::Collection<Document>,
    timer_duration: &str,
    page: &str,
    limit: &str,
) -> Result<Vec<LeaderboardEntry>, mongodb::error::Error> {
    let pipeline = create_aggregation_pipeline(timer_duration, page, limit);

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

fn create_aggregation_pipeline(timer_duration: &str, page: &str, limit: &str) -> Vec<Document> {
    let page_number: usize = page.parse().unwrap_or(1);
    let limit_number: usize = limit.parse().unwrap_or(10);
    let skip_number = (page_number - 1) * limit_number;

    vec![
        // Match documents where the field for the given timer duration exists
        doc! { "$match": { format!("high_scores.{}", timer_duration): { "$exists": true } } },
        // Project the necessary fields, including the WPM for the given timer duration
        doc! { "$project": {
            "_id": 1,
            "username": 1,
            "completed_tests": 1,
            format!("high_scores.{}", timer_duration): 1
        }},
        // Skip documents based on the page number and limit
        doc! { "$skip": skip_number as i64 },
        doc! { "$limit": limit_number as i64 },
        // Sort by the best WPM in descending order
        doc! { "$sort": { format!("high_scores.{}.wpm", timer_duration): -1 } },
    ]
}

// fn log_leaderboard_stats(users: &[LeaderboardEntry]) {
//     println!(
//         "Leaderboard Stats: {}",
//         serde_json::to_string_pretty(&json!(users)).unwrap()
//     );
// }
