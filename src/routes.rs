use rocket::State;
use serde::Serialize;
use rocket::http::Status;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use rocket_contrib::json::Json;
use std::collections::{HashMap, HashSet};

type Database = Arc<Mutex<HashMap<String, HashSet<String>>>>;

#[get("/<domain>/url")]
pub fn list(cache: State<Database>, domain: String) -> Result<Json<Vec<String>>, Status> {
    match cache.lock() {
        // If lock was aquired, find domain in the list and return all items from the set as
        // vector or an empty vector if domain was not scraped yet.
        Ok(db) => {
            let list = match db.get(&domain) {
                Some(set) => set.iter().cloned().collect(),
                None => vec!(),
            };

            Ok(Json(list))
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/<domain>/url/count")]
pub fn count(cache: State<Database>, domain: String) -> Result<Json<UrlCount>, Status> {
    match cache.lock() {
        // If lock was aquired, find domain and count all urls it has associated with it.
        Ok(db) => {
            let count: usize = match db.get(&domain) {
                Some(set) => set.len(),
                None => 0,
            };

            Ok(Json(UrlCount { count }))
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/", data = "<url>")]
pub fn scrape(producer: State<Mutex<Sender<String>>>, url: String) -> Status {
    // TODO: Find a better way of creating a channel without using mutex.
    // TODO: Handle mutex error.
    match producer.lock().unwrap().send(url.clone()) {
        Ok(_) => Status::Accepted,
        Err(_) => Status::ServiceUnavailable,
    }
}

#[derive(Serialize)]
pub struct UrlCount {
    /// How many unique urls has the scraper found for given domain.
    count: usize
}
