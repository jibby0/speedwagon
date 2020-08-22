use crate::schema::users;
use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use time;
use uuid::Uuid;

enum SourceData {
    RSS {url: String}
}

#[derive(Queryable, AsChangeset, Serialize, Deserialize, Debug, Identifiable, Insertable)]
#[table_name = "sources"]
#[primary_key("id")]
#[belongs_to(User, foreign_key = "creator")]
pub struct Source {
    pub id: Uuid,
    pub title: String,
    pub data: SourceData,
    pub filter: String,
    pub last_post: time::Timespec,
    pub last_successful_fetch: time::Timespec,
    pub fetch_errors: [String],
    pub creator: String,
    // TODO optional config line for sharing
    // TODO optional config arg to make copies on Source changes, on untrusted servers
    pub public: bool
}

pub fn all_from_user(username: String, connection: &PgConnection) -> QueryResult<Vec<Source>> {
    sources::table.filter(creator.eq(username)).load::<Source>(&*connection)
}

pub fn all_public(connection: &PgConnection) -> QueryResult<Vec<Source>> {
    sources::table.filter(public.eq(true)).load::<Source>(&*connection)
}

pub fn get(id: Uuid, connection: &PgConnection) -> QueryResult<Source> {
    users::table.find(id).get_result::<Source>(connection)
}

pub fn insert(source: Source, connection: &PgConnection) -> QueryResult<Source> {
    diesel::insert_into(sources::table)
        .values(source)
        .get_result(connection)
}

pub fn update(source: Source, connection: &PgConnection) -> QueryResult<Source> {
    diesel::update(sources::table.find(source.id.clone()))
        .set(source)
        .get_result(connection)
}

pub fn delete(id: Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(sources::table.find(id))
        .execute(connection)
}

