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
                scrape_urls(&db, url.as_str().to_string(), url.host_str().unwrap());
            },
            None => continue,
        }
    }
}

/// Scrapes given url and finds all link that are of the same hostname. It then visits the links
/// looking for move unique links. Once it drains all usable links on given hostname, it stops 
/// crawling.
fn scrape_urls(master: &Database, url: String, host: &str) {
    let mut queue: Vec<String> = vec!(url);

    loop {
        if queue.len() == 0 {
            break;
        }

        // Unwrap is safe here as we just checked for the length.
        let scraped_urls = crawl(host, queue.pop().unwrap()); 

        // Appends all unique urls found on given site. 
        queue.append(
            &mut insert_unique_urls(master, scraped_urls, host)
        );
    }
}

fn crawl(host: &str, url: String) -> HashSet<String> {
    HashSet::new()
}

/// Compares the set of scraped urls against the database, inserts the new ones and returns them.
fn insert_unique_urls(master: &Database, mut scraped_urls: HashSet<String>, host: &str) -> Vec<String> {
    // Aquires the database lock.
    // TODO: Error handling the mutex.
    let mut map = master.lock().unwrap();

    // Gets the HashSet associated with given domain.
    match map.get_mut(host) {
        // If the domain has been already crawled, adds new items to the set.
        Some(set) => {
            let mut unique_urls: Vec<String> = Vec::new();

            // For each newly scraped url, tries to insert it into the HashSet.
            // If the url hasn't been in the set prior, pushes it into unique
            // urls collections to be crawled in next cycle.
            for url in scraped_urls.drain() {
                if (*set).insert(url.clone()) {
                    unique_urls.push(url);
                }
            }

            unique_urls
        },
        // If the domain hasn't been crawled yet, all scraped urls are unique.
        None => {
            // Copies the urls so that they can be crawled in next cycle.
            let items: Vec<String> = scraped_urls.iter().cloned().collect();

            // Inserts the HashSet into the database.
            map.insert(host.to_string(), scraped_urls);

            items
        }
    }
}


