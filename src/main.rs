#![feature(proc_macro_hygiene, decl_macro)]

extern crate url;
extern crate serde;
#[macro_use]
extern crate rocket;
extern crate scraper;
extern crate reqwest;
extern crate rocket_contrib;

mod routes;
mod crawler;

use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::collections::{HashMap, HashSet};

type Database = Arc<Mutex<HashMap<String, HashSet<String>>>>;

fn main() {
  // Creates an empty hash map where the domains and their unique urls are stored.
  let db: Database = Default::default();
  let cache: Database = Arc::clone(&db);

  // Creates a channel on which the POST /host/${domain} endpoint commits new domains to crawler.
  let (producer, consumer) = channel::<String>();

  // Creates a new thread on which the crawler runs. It has access to the database to which it
  // commits new urls and to the consumer half of the channel.
  thread::spawn(move || crawler::listen(db, consumer));

  // Starts the web server with crawl, list and count routes. Also adds url cache and crawler
  // channel to the container to be used by the endpoints.
  rocket::ignite()
    .mount("/host", routes![routes::crawl, routes::list, routes::count])
    .manage(cache)
    // TODO: Find a better way of sharing producer channel handle without mutex since rocket
    // can't move .clone().
    .manage(Mutex::new(producer))
    .launch();
}
