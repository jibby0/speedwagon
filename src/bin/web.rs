extern crate speedwagon;
#[macro_use]
extern crate rocket;

use dotenv;
use speedwagon::{
    api::v1::{items, users},
    db, logger, state,
};

use rocket::fairing::AdHoc;

fn main() {
    dotenv::dotenv().ok();
    logger::setup_logging(log::LevelFilter::Debug)
        .expect("failed to initialize logging");
    rocket::ignite()
        .manage(db::init_pool())
        .mount(
            "/",
            routes![
                items::index,
                users::user_create,
                users::user_login,
                users::user_logout,
                users::user_index,
            ],
        )
        .attach(AdHoc::on_attach("Environment tracker", |rocket| {
            let env = rocket.config().environment.clone();
            Ok(rocket.manage(state::Environment(env)))
        }))
        .launch();
}
