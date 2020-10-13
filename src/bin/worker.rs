extern crate clokwerk;
extern crate speedwagon;

use clokwerk::{Scheduler, TimeUnits};

use speedwagon::{db, fetch, logger};

fn main() {
    dotenv::dotenv().ok();
    logger::setup_logging(log::LevelFilter::Debug)
        .expect("failed to initialize logging");

    let mut pool = db::init_pool();
    let f = move || fetch::fetch_new_from_all_sources(&mut pool);
    let mut scheduler = Scheduler::new();
    scheduler.every(10.minutes()).run(f);
}
