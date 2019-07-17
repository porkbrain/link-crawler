use rocket::State;
use serde::Serialize;
use rocket_contrib::json::Json;
use std::collections::{HashMap, HashSet};

type Cache = HashMap<String, HashSet<String>>;

#[derive(Serialize)]
pub struct UrlCount {
    /// How many unique urls has the scraper found for given domain.
    count: usize
}

#[get("/<domain>/url/count")]
pub fn count(cache: State<Cache>, domain: String) -> Json<UrlCount> {
    let count: usize = match cache.get(&domain) {
        Some(set) => set.len(),
        None => 0,
    };

    Json(UrlCount { count })
}

