# Link scraper

A Rust coding challenge project. Runs web server which exposes three endpoints. The web server crawls given domain and caches links it found within the same host name.

## POST /host

Expects body to be a raw string valid URL to crawl.

_RESPONSE_

* `202` if url was scheduled to be crawled.
* `500` if an unexpected error happened during job scheduling.

## GET /host/{domain}/url

Lists unique urls crawled for given domain. Domain has to be a valid hostname in format `optional-subdomain.example.com`. The crawler makes a distinction between `test.example.com` and `example.com`.

_RESPONSE_

* `200` with a JSON array of strings.
* `503` if a lock to the database was not acquired.

## GET /host/{domain}/url/count

Counts unique urls crawled for given domain. In the body returns a raw numeric string.

_RESPONSE_

* `200` with a raw numeric string.
* `503` if a lock to the database was not acquired.
