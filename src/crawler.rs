use url::Url;
use std::sync::{Arc, Mutex};
use scraper::{Html, Selector};
use std::sync::mpsc::Receiver;
use std::collections::{HashMap, HashSet};

type Database = Arc<Mutex<HashMap<String, HashSet<String>>>>;

/// Crawler assumes its own thread as it blocks. It listen to consumer channel for url.
/// It checks the domain and makes a request to it. It collects recursivelly all urls it can
/// find that belong to the same hostname. These urls are then stored in a HashSet and commited
/// to the database.
pub fn listen(db: Database, consumer: Receiver<String>) {
  loop {
    // Waits for a message to be sent down the channel.
    let message = consumer.recv();

    if message.is_err() {
      // Prints out the error and awaits next message.
      println!("[Crawler] Error during message receiving: {:?}", message.err().unwrap());

      continue;
    }

    // Unwrap here is safe as we have just checked for error. We can ignore the error given
    // by parsing as a malformed url is a user, not server error.
    match Url::parse(&message.unwrap()).ok().filter(|url| url.has_host()) {
      Some(url) => {
        // Unwrap here is safe as we have filtered `has_host` in match statement.
        crawl_urls(&db, url.as_str().to_string(), url.host_str().unwrap());
      },
      None => continue,
    }
  }
}

/// Crawls given url and finds all link that are of the same hostname. It then visits the links
/// looking for move unique links. Once it drains all usable links on given hostname, it stops
/// crawling.
fn crawl_urls(master: &Database, url: String, host: &str) {
  let mut queue: Vec<String> = vec!(url);

  loop {
    if queue.len() == 0 {
      break;
    }

    // Unwrap is safe here as we just checked for the length.
    if let Some(crawled_urls) = crawl(host, queue.pop().unwrap()) {
      // Appends all unique urls found on given site.
      queue.append(
        &mut insert_unique_urls(master, crawled_urls, host)
      );
    }

  }
}

/// Crawls all urls on given website and filters out the ones not belonging to given
/// host name. HashSet also makes sure all returned urls are unique.
fn crawl(host: &str, url: String) -> Option<HashSet<String>> {
  let mut req = reqwest::get(&url).ok()?;

  if !req.status().is_success() {
    return None;
  }

  let body = req.text().ok()?;
  let dom = Html::parse_document(&body);
  let link_selector = Selector::parse("a").ok()?;

  // Finds all links in the DOM and filters them based on host name.
  let mut urls: HashSet<String> = dom.select(&link_selector)
    .filter_map(|node| {
      let link = node.value().attr("href")?;

      // Checks the hostname to ensure the links are from a single domain.
      // TODO: If the url starts with forward slash, prepend the host name to it.
      let link_host = Url::parse(link).ok()?.host_str()?.to_string();

      if *host != link_host {
        return None;
      }

      // More checks or sanitization could be done here, such as stripping query parameters.

      Some(link.to_string())
    })
    .collect();

  urls.insert(url);

  Some(urls)
}

/// Compares the set of crawled urls against the database, inserts the new ones and returns them.
fn insert_unique_urls(master: &Database, mut crawled_urls: HashSet<String>, host: &str) -> Vec<String> {
  // Acquires the database lock.
  // TODO: Error handling the mutex.
  let mut map = master.lock().unwrap();

  // Gets the HashSet associated with given domain.
  match map.get_mut(host) {
    // If the domain has been already crawled, adds new items to the set.
    Some(set) => {
      let mut unique_urls: Vec<String> = Vec::new();

      // For each newly crawled url, tries to insert it into the HashSet.
      // If the url hasn't been in the set prior, pushes it into unique
      // urls collections to be crawled in next cycle.
      for url in crawled_urls.drain() {
        if (*set).insert(url.clone()) {
          unique_urls.push(url);
        }
      }

      unique_urls
    },
    // If the domain hasn't been crawled yet, all crawled urls are unique.
    None => {
      // Copies the urls so that they can be crawled in next cycle.
      let items: Vec<String> = crawled_urls.iter().cloned().collect();

      // Inserts the HashSet into the database.
      map.insert(host.to_string(), crawled_urls);

      items
    }
  }
}


