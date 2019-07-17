#![feature(proc_macro_hygiene, decl_macro)]

extern crate serde;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use std::collections::{HashMap, HashSet};

mod list;
mod count;
mod scrape;

type Cache = HashMap<String, HashSet<String>>;

fn main() {
    let mut cache: Cache = HashMap::new();
    let mut set: HashSet<String> = HashSet::new();
    set.insert(String::from("test.com/pls"));
    cache.insert("test.com".to_string(), set);

    rocket::ignite()
        .mount("/host", routes![scrape::scrape, list::list, count::count])
        .manage(cache)
        .launch();
}
