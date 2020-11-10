extern crate speedwagon;
#[macro_use]
extern crate rocket;

use speedwagon::{
    api::v1::{items, sources, users},
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
            "/api/v1/",
            routes![
                items::index,
                users::user_create,
                users::user_login,
                users::user_logout,
                users::user_index,
                sources::source_create,
                sources::sources_list,
                sources::source_update,
                sources::source_delete,
            ],
        )
        .attach(AdHoc::on_attach("Environment tracker", |rocket| {
            let env = rocket.config().environment;
            Ok(rocket.manage(state::Environment(env)))
        }))
        .launch();
}
