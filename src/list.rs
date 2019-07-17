use rocket::State;
use rocket_contrib::json::Json;
use std::collections::{HashMap, HashSet};

type Cache = HashMap<String, HashSet<String>>;

#[get("/<domain>/url")]
pub fn list(cache: State<Cache>, domain: String) -> Json<Vec<String>> {
    match cache.get(&domain) {
        Some(set) => Json(set.iter().cloned().collect()),
        None => Json(vec!()),
    }
}

