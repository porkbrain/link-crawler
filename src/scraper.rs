use url::Url;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::collections::{HashMap, HashSet};

type Database = Arc<Mutex<HashMap<String, HashSet<String>>>>;

/// Scraper assumes its own thread as it blocks. It listen to consumer channel for url.
/// It checks the domain and makes a request to it. It collects recursivelly all urls it can
/// find that belong to the same hostname. These urls are then stored in a HashSet and commited
/// to the database.
pub fn listen(db: Database, consumer: Receiver<String>) {
    loop {
        // Waits for a message to be sent down the channel.
        let message = consumer.recv();

        if message.is_err() {
            // Prints our the error and awaits next message.
            println!("[Scraper] Error during message receiving: {:?}", message.err().unwrap());

            continue;
        }

        // Unwrap here is safe as we have just checked for error. We can ignore the error given
        // by parsing as a malformed url is a user, not server error.
        match Url::parse(&message.unwrap()).ok().filter(|url| url.has_host()) {
            Some(url) => {
                // Unwrap here is safe as we have filtered `has_host` in match statement.
                scrape_urls(&db, url.as_str(), url.host_str().unwrap());
            },
            None => continue,
        }
    }
}

fn scrape_urls(master: &Database, url: &str, host: &str) {
    println!("[{}] Scraping {}", host, url);

    let scraped: HashSet<String> = HashSet::new();

    let _diff: Vec<String> = {
        let mut map = master.lock().unwrap();

        match map.get_mut(host) {
            Some(set) => {
                let difference: Vec<String> = scraped.difference(set).cloned().collect();
                let mut items: Vec<String> = Vec::new();

                for url in difference {
                    (*set).insert(url.clone());
                    items.push(url.to_string());
                }

                items
            },
            None => {
                let items: Vec<String> = scraped.iter().cloned().collect();

                map.insert(host.to_string(), scraped);

                items
            }
        }
    };
}


