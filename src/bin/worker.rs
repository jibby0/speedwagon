extern crate speedwagon;
extern crate clokwerk;

use clokwerk::{Scheduler, TimeUnits};

use speedwagon::logger;

fn main() {
    dotenv::dotenv().ok();
    logger::setup_logging(log::LevelFilter::Debug)
        .expect("failed to initialize logging");

    let mut scheduler = Scheduler::new();
    scheduler.every(10.minutes()).run();
    print!("I am worker");
}
