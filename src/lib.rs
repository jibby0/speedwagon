#![feature(proc_macro_hygiene, decl_macro)]
extern crate chrono;

#[macro_use]
extern crate rocket;
extern crate bcrypt;
extern crate reqwest;
extern crate rfc822_sanitizer;
extern crate rocket_contrib;
extern crate time;
#[macro_use]
extern crate diesel;

pub mod api;
pub mod db;
pub mod fetch;
pub mod logger;
pub mod schema;
pub mod sources;
pub mod state;
pub mod timestamp;
pub mod setup_rocket;

use std::{error::Error, result::Result as StdResult};
type Result<T> = StdResult<T, Box<dyn Error>>;
