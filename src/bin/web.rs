extern crate speedwagon;
#[macro_use]
extern crate rocket;

use speedwagon::{
    api::v1::{items, sources, users},
    db, logger, state, setup_rocket::setup_rocket
};



fn main() {
    logger::setup_logging(log::LevelFilter::Debug)
        .expect("failed to initialize logging");
    setup_rocket().launch();
}
