use rocket::request::{Request, FromRequest, Outcome};
use rocket::State;
use rocket::http::Status;
use crate::speedwagon::api::v1::users::User;
use crate::speedwagon::db::{DbConn, Pool};
use crate::speedwagon::schema::users;
use crate::speedwagon::db::tokens;
use diesel::prelude::*;

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<User, Self::Error> {
        let token = match request.cookies()
            .get_private("api_token")
            .and_then(|cookie| cookie.value().parse().ok()) {
                Some(c) => c,
                None => return Outcome::Forward(()),
            };

        let pool = request.guard::<State<Pool>>()?;
        let conn = match pool.get() {
            Ok(conn) => DbConn(conn),
            Err(_) => return Outcome::Failure((Status::ServiceUnavailable, ())),
        };

        let username = match tokens::get(token, &conn) {
            Err(_) => return Outcome::Failure((Status::Unauthorized, ())),
            Ok(token) => token.username
        };

        match get(username, &conn) {
            Err(_) => Outcome::Failure((Status::Gone, ())),
            Ok(user) => Outcome::Success(user)
        }
    }
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<User>> {
    users::table.load::<User>(&*connection)
}

pub fn get(username: String, connection: &PgConnection) -> QueryResult<User> {
    users::table.find(username).get_result::<User>(connection)
}

pub fn insert(user: User, connection: &PgConnection) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(user)
        .get_result(connection)
}

pub fn update(user: User, connection: &PgConnection) -> QueryResult<User> {
    diesel::update(users::table.find(user.username.clone()))
        .set(user)
        .get_result(connection)
}

pub fn delete(username: String, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(users::table.find(username))
        .execute(connection)
}
