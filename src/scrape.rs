use rocket::State;
use rocket::response::status;

#[post("/<domain>")]
pub fn scrape(domain: String) -> status::Accepted::<()> {
    status::Accepted(None)
}
