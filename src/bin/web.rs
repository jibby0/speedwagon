extern crate speedwagon;
#[macro_use]
extern crate rocket;

use speedwagon::{
    api::v1::{items, sources, users},
    db, logger, state,
};

use rocket::fairing::AdHoc;

fn main() {
    logger::setup_logging(log::LevelFilter::Debug)
        .expect("failed to initialize logging");
    setup_rocket().launch();
}

// TODO move this to a shared spot
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
}

// TODO move this to api/v1/users
#[cfg(test)]
mod tests {
    use super::*;
    use rocket::{http::Status, local::Client};

    #[test]
    fn api_user_create() {
        let client =
            Client::new(setup_rocket()).expect("valid rocket instance");
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, world!".into()));
    }
}
