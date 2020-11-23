extern crate speedwagon;

use speedwagon::{logger, setup_rocket::setup_rocket};

fn main() {
    logger::setup_logging(log::LevelFilter::Debug)
        .expect("failed to initialize logging");
    setup_rocket().launch();
}
