use rocket::request::{Request, FromRequest, Outcome};
use uuid::Uuid;
use rocket::State;
use rocket::http::{ContentType, Status};
use crate::speedwagon::api::v1::users::{User, ValidToken};
use crate::speedwagon::db::{DbConn, Pool};
use crate::speedwagon::schema::users;
use crate::speedwagon::db::tokens;
use diesel::prelude::*;

impl<'a, 'r> FromRequest<'a, 'r> for ValidToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<ValidToken, Self::Error> {
        let opt_token = request.headers().get_one("Authorization").and_then(
            |bearer| {
                let v: Vec<&str> = bearer.split_ascii_whitespace().collect();
                match v[..] {
                    [_, token] => Uuid::parse_str(token).ok().and_then(
                        |token| {
                            log::debug!("Found header token {}", token);
                            Some(token)
                        }),
                    _ => None,
                }
            }
        ).or_else(
            || request.cookies()
                .get_private("api_token")
                .and_then(|cookie| cookie.value().parse().ok())
                .and_then(|token| {
                            log::debug!("Found cookie token {}", token);
                            Some(token)
                        }
        ));
        let token_id = match opt_token {
            Some(token) => token,
            None => return Outcome::Forward(()),
        };

        let pool = request.guard::<State<Pool>>()?;
        let conn = match pool.get() {
            Ok(conn) => DbConn(conn),
            Err(_) => return Outcome::Failure((Status::ServiceUnavailable, ())),
        };

        let token = match tokens::get(token_id, &conn) {
            Ok(token) => token,
            Err(_) => return Outcome::Failure((Status::Unauthorized, ())),
        };
        if token.expires < time::now().to_timespec() {
            return Outcome::Failure((Status::Unauthorized, ()))
        }

        Outcome::Success(ValidToken{id: token.id, username: token.username})
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
