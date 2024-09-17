use crate::constants::{COLL_NAME, DB_NAME};
use mongodb::bson::Document;
use mongodb::{Client, Collection};

pub fn get_collection(client: &Client) -> Collection<Document> {
    client.database(DB_NAME).collection(COLL_NAME)
}
