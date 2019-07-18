# Link Crawler

A Rust coding challenge project. Runs web server which exposes three endpoints. The web server crawls given domain and caches links it found within the same host name.

Tested with `rustc 1.38.0-nightly (07e0c3651 2019-07-16)` and `cargo 1.38.0-nightly (677a180f4 2019-07-08)`. This project will compile only with nightly build. To update to nightly, run `rustup update nightly`.

## Docker

The provided *Dockerfile* is ready to be deployed to an AWS EC2 (might just need changing the port). However, the app can be run locally too (you might need to change the path in the *Dockerfile*):

```
$ docker build -t crawler .
$ docker run -it -p 127.0.0.1:8000:8000 --rm --name crawler-instance crawler
```

## Endpoints

### POST /host

Expects body to be a JSON including a valid URL to crawl.

_REQUEST_

```
{
	"url": "https://example.com"
}
```

_RESPONSE_

* `202` if url was scheduled to be crawled.
* `500` if an unexpected error happened during job scheduling.

### GET /host/{domain}/url

Lists unique urls crawled for given domain. Domain has to be a valid hostname in format `optional-subdomain.example.com`. The crawler makes a distinction between `test.example.com` and `example.com`.

_RESPONSE_

* `200` with a JSON array of strings.
* `503` if a lock to the database was not acquired.

### GET /host/{domain}/url/count

Counts unique urls crawled for given domain. In the body returns a raw numeric string.

_RESPONSE_

* `200` with a raw numeric string.
* `503` if a lock to the database was not acquired.
