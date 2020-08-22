use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable,
    AsChangeset,
    Serialize,
    Deserialize,
    Debug,
    Identifiable,
    Insertable,
)]
#[table_name = "users"]
#[primary_key("username")]
pub struct User {
    pub username: String,
    pub password: String,
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

pub fn delete(
    username: String,
    connection: &PgConnection,
) -> QueryResult<usize> {
    diesel::delete(users::table.find(username)).execute(connection)
}
