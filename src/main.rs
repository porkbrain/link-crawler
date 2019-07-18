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
use rocket::Rocket;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::collections::{HashMap, HashSet};

pub type Database = Arc<Mutex<HashMap<String, HashSet<String>>>>;

fn main() {
  // Creates an empty hash map where the domains and their unique urls are stored.
  let db: Database = Default::default();
  let cache: Database = Arc::clone(&db);

  // Creates a channel on which the POST /host/${domain} endpoint commits new domains to crawler.
  let (producer, consumer) = channel::<String>();

  // Creates a new thread on which the crawler runs. It has access to the database to which it
  // commits new urls and to the consumer half of the channel.
  thread::spawn(move || crawler::listen(db, consumer));

  server(cache, producer).launch();
}

/// Starts the web server with crawl, list and count routes. Also adds url cache and crawler
/// channel to the container to be used by the endpoints.
fn server(cache: Database, producer: Sender<String>) -> Rocket {
  rocket::ignite()
    .mount("/host", routes![routes::crawl, routes::list, routes::count])
    .manage(cache)
    // TODO: Find a better way of sharing producer channel handle without mutex since rocket
    // can't move .clone().
    .manage(Mutex::new(producer))
}

#[cfg(test)]
mod test {
  use std::thread;
  use super::server;
  use super::rocket;
  use super::Database;
  use rocket::http::Status;
  use rocket::local::Client;
  use std::sync::mpsc::channel;
  use std::collections::HashSet;
  use rocket::http::ContentType;

  #[test]
  fn test_list_without_urls() {
    // Creates an empty database.
    let db: Database = Default::default();

    let (producer, _) = channel::<String>();

    let client = Client::new(server(db, producer)).unwrap();
    let mut response = client.get("/host/github.com/url").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("[]".into()));
  }

  #[test]
  fn test_list_with_urls() {
    // Inserts a link into the database.
    let db: Database = Default::default();
    let mut set = HashSet::new();
    set.insert(String::from("https://github.com"));
    db.lock().unwrap().insert("github.com".to_string(), set);

    let (producer, _) = channel::<String>();

    let client = Client::new(server(db, producer)).unwrap();
    let mut response = client.get("/host/github.com/url").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("[\"https://github.com\"]".into()));
  }

  #[test]
  fn test_count_without_urls() {
    // Creates an empty database.
    let db: Database = Default::default();

    let (producer, _) = channel::<String>();

    let client = Client::new(server(db, producer)).unwrap();
    let mut response = client.get("/host/github.com/url/count").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("{\"count\":0}".into()));
  }

  #[test]
  fn test_count_with_urls() {
    // Inserts a link into the database.
    let db: Database = Default::default();
    let mut set = HashSet::new();
    set.insert(String::from("https://github.com"));
    db.lock().unwrap().insert("github.com".to_string(), set);

    let (producer, _) = channel::<String>();

    let client = Client::new(server(db, producer)).unwrap();
    let mut response = client.get("/host/github.com/url/count").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("{\"count\":1}".into()));
  }

  #[test]
  fn test_crawl() {
    let db: Database = Default::default();

    let (producer, consumer) = channel::<String>();

    let client = Client::new(server(db, producer)).unwrap();

    let receiver_test = thread::spawn(move || {
      assert_eq!("https://github.com", consumer.recv().unwrap());
    });

    let response = client
      .post("/host")
      .header(ContentType::JSON)
      .body("{\"url\":\"https://github.com\"}")
      .dispatch();

    assert_eq!(response.status(), Status::Accepted);
    receiver_test.join().unwrap();
  }

  // TODO: Test cases for crawler.

}
