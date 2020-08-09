use crate::speedwagon::schema::users;
use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use time;

enum SourceData {
    RSS {url: String}
}

#[derive(Queryable, AsChangeset, Serialize, Deserialize, Debug, Identifiable, Insertable)]
#[table_name = "sources"]
#[primary_key("creator", "title")]
#[belongs_to(User, foreign_key = "creator")]
pub struct Source {
    pub title: String,
    pub data: SourceData,
    pub url: String,
    pub last_post: time::Timespec,
    pub last_successful_fetch: time::Timespec,
    pub fetch_errors: [String],
    pub creator: String,
    pub public: bool
}

pub fn all_from_user(username: String, connection: &PgConnection) -> QueryResult<Vec<Source>> {
    sources::table.filter(creator.eq(username)).load::<Source>(&*connection)
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

