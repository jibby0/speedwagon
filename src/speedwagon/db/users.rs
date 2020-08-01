use crate::speedwagon::api::v1::users::{User};
use crate::speedwagon::schema::users;
use diesel::prelude::*;

#[derive(Insertable)]
#[table_name = "users"]
struct InsertableUser {
    username: String,
    password: String
}

impl InsertableUser {
    fn from_user(user: User) -> InsertableUser {
        InsertableUser {
            username: user.username,
            password: user.password,
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
        .values(&InsertableUser::from_user(user))
        .get_result(connection)
}

pub fn update(user: User, connection: &PgConnection) -> QueryResult<User> {
    diesel::update(users::table.find(user.username))
        .set(&user)
        .get_result(connection)
}

pub fn delete(username: String, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(users::table.find(username))
        .execute(connection)
}
