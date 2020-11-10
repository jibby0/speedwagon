extern crate clokwerk;
extern crate speedwagon;

use clokwerk::{Scheduler, TimeUnits};

use speedwagon::{db, fetch, logger};

fn main() {
    dotenv::dotenv().ok();
    logger::setup_logging(log::LevelFilter::Debug)
        .expect("failed to initialize logging");

    let mut pool = db::init_pool();
    // TODO would an "update_requested" flag on each source be better?
    // A background worker could then do these pulls in parallel,
    //  and another task sets "update_requested=True" on each source
    //  every ~10 mins.
    let f = move || {
        if let Err(e) = fetch::fetch_new_from_all_sources(&mut pool) {
            log::error!("{}", e);
        }
    };
    let mut scheduler = Scheduler::new();
    scheduler.every(10.minutes()).run(f);
}
