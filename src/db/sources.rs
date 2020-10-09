use crate::{
    db::{tagged_sources::TaggedSource, tags::Tag, users::User},
    schema::{sources, tagged_sources},
    sources::rssatom,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use time;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
enum SourceData {
    RSSAtom(rssatom::RSSAtom),
}

#[derive(
    Associations, Queryable, AsChangeset, Debug, Identifiable, Insertable,
)]
#[table_name = "sources"]
#[belongs_to(User, foreign_key = "creator")]
pub struct Source {
    pub id: Uuid,
    pub title: String,
    // TODO from_value should be called on this before handing it to the API
    // side.
    pub source_data: serde_json::Value,
    pub post_filter: String,
    pub last_post: time::Timespec,
    pub last_successful_fetch: time::Timespec,
    pub fetch_errors: Vec<String>,
    pub creator: String,
    // TODO optional config line for sharing
    // TODO optional config arg to make copies on Source changes, on untrusted
    // servers
    pub public: bool,
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<Source>> {
    sources::table.load::<Source>(&*connection)
}

pub fn all_from_user(
    username: String,
    connection: &PgConnection,
) -> QueryResult<Vec<Source>> {
    sources::table
        .filter(sources::creator.eq(username))
        .load::<Source>(&*connection)
}

pub fn all_public(connection: &PgConnection) -> QueryResult<Vec<Source>> {
    sources::table
        .filter(sources::public.eq(true))
        .load::<Source>(&*connection)
}

pub fn all_from_tag(
    tag: Tag,
    connection: &PgConnection,
) -> QueryResult<Vec<Source>> {
    tagged_sources::table
        .filter(tagged_sources::tag.eq(tag.id))
        .inner_join(sources::table)
        .load::<(TaggedSource, Source)>(&*connection)
        // TODO There's probably a more DSL-oriented way to transform
        // (TaggedSource, Source) -> Source.
        .and_then(|v| Ok(v.into_iter().map(|(_tagged_src, src)| src).collect()))
}

pub fn get(id: Uuid, connection: &PgConnection) -> QueryResult<Source> {
    sources::table.find(id).get_result::<Source>(connection)
}

pub fn insert(
    source: Source,
    connection: &PgConnection,
) -> QueryResult<Source> {
    diesel::insert_into(sources::table)
        .values(source)
        .get_result(connection)
}

pub fn update(
    source: Source,
    connection: &PgConnection,
) -> QueryResult<Source> {
    diesel::update(sources::table.find(source.id.clone()))
        .set(source)
        .get_result(connection)
}

pub fn delete(id: Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(sources::table.find(id)).execute(connection)
}
