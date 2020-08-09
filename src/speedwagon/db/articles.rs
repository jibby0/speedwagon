use crate::speedwagon::schema::users;
use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use time;

#[derive(Queryable, AsChangeset, Serialize, Deserialize, Debug, Identifiable, Insertable)]
#[table_name = "articles"]
// #[primary_key("username")]
pub struct Article {
    pub title: String,
    pub content: String,
    pub date: time::Timespec,
    // TODO more
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


