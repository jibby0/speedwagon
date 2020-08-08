#![feature(proc_macro_hygiene, decl_macro)]

use std::io;

mod speedwagon;
use speedwagon::api::v1::{items, users};
use speedwagon::state;
use speedwagon::db;
use dotenv;

extern crate chrono;

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate reqwest;
extern crate time;
extern crate bcrypt;
#[macro_use] extern crate diesel;
use rocket::fairing::AdHoc;


fn main() {
    dotenv::dotenv().ok();
    setup_logging(log::LevelFilter::Debug).expect("failed to initialize logging");
    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![
            items::index,
            users::create,
            users::login,
            users::logout,
            users::user_index,
        ])
        .attach(AdHoc::on_attach("Environment tracker", |rocket| {
            let env = rocket.config().environment.clone();
            Ok(rocket.manage(state::Environment(env)))
        }))
        .launch();
}

fn setup_logging(verbosity: log::LevelFilter) -> Result<(), fern::InitError> {
    let base_config = fern::Dispatch::new().level(verbosity);

    let stdout_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(io::stdout());

    base_config
        .chain(stdout_config)
        .apply()?;

    Ok(())
}
