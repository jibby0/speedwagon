use rss::Channel;
use atom_syndication::Feed;
use std::io::BufReader;

use log::debug;

extern crate reqwest;

#[get("/api/v1/items")]
pub fn index() -> String {
    let resp = reqwest::blocking::get("https://www.benningtonbanner.com/category/local-news/browse.xml")
        .unwrap().text().unwrap();

    match Channel::read_from(BufReader::new(resp.as_bytes())) {
        Ok(content) => return content.to_string(),
        Err(e) => debug!("RSS Channel parse failed: {}", e),
    }

    match Feed::read_from(BufReader::new(resp.as_bytes())) {
        Ok(content) => {
            debug!("{}", content.to_string());
            return content.to_string()},
        Err(e) => debug!("Atom Feed parse failed: {}", e),
    }

    String::from("No format was compatable")
}
