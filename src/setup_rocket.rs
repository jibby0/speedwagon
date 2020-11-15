use crate::{
    api::v1::{items, sources, users},
    db, state,
};

use rocket::fairing::AdHoc;

pub fn setup_rocket() -> rocket::Rocket {
    dotenv::dotenv().expect("Failed to read .env file");
    rocket::ignite()
        .manage(db::init_pool())
        .mount(
            "/api/v1/",
            routes![
                items::index,
                users::user_create,
                users::user_login,
                users::user_change_pass,
                users::user_logout,
                users::user_delete,
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
}

