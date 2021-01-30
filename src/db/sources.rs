use crate::{
    db::{tagged_sources::TaggedSource, tags::Tag, users::User},
    schema::{sources, tagged_sources},
    sources::rssatom,
    timestamp::Timestamp,
};
use chrono::Duration;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum SourceData {
    RSSAtom(rssatom::RSSAtom),
}

#[derive(
    Associations,
    Queryable,
    AsChangeset,
    Debug,
    Identifiable,
    Insertable,
    Serialize,
    Deserialize,
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
    pub last_post: Timestamp,
    pub last_successful_fetch: Timestamp,
    pub fetch_errors: Vec<String>,
    pub creator: String,
    pub fetching: bool,
    pub last_fetch_started: Timestamp,
    /* TODO optional config line for sharing
     * TODO optional config arg to make copies on Source changes, on
     * untrusted servers */
}

impl Source {
    pub fn new(
        id: Option<Uuid>,
        title: String,
        source_data: serde_json::Value,
        post_filter: String,
        creator: String,
    ) -> Source {
        Source {
            id: id.unwrap_or_else(Uuid::new_v4),
            title,
            source_data,
            post_filter,
            creator,
            last_successful_fetch: Timestamp::now(),
            last_post: Timestamp::now(),
            fetch_errors: Vec::new(),
            fetching: false,
            last_fetch_started: Timestamp::now(),
        }
    }
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

pub fn all_from_tag(
    tag: Tag,
    connection: &PgConnection,
) -> QueryResult<Vec<Source>> {
    tagged_sources::table
        .filter(tagged_sources::tag.eq(tag.id))
        .inner_join(sources::table)
        .load::<(TaggedSource, Source)>(&*connection)
        .map(|v| v.into_iter().map(|(_tagged_src, src)| src).collect())
}

pub fn get(id: Uuid, connection: &PgConnection) -> QueryResult<Source> {
    sources::table.find(id).get_result::<Source>(connection)
}

/// Get all sources that need to be fetched.
///
/// `get`, but checks & sets `fetching=true` & last_fetch_started.
/// The client is responsible for setting  `fetching=false` and
/// last_successful_fetch upon success.
pub fn get_for_fetch(connection: &PgConnection) -> QueryResult<Vec<Source>> {
    let this_fetch = Timestamp::now();
    connection.transaction(|| {
        let mut sources = sources::table
            .for_update()
            .filter(
                sources::fetching
                    .eq(false)
                    .and(
                        sources::last_successful_fetch
                            .le(this_fetch - Duration::minutes(15)),
                    )
                    .or(sources::fetching.eq(true).and(
                        sources::last_fetch_started
                            .le(this_fetch - Duration::minutes(15)),
                    )),
            )
            .load::<Source>(&*connection)?;

        for source in &mut sources {
            source.fetching = true;
            source.last_fetch_started = this_fetch;
            update(source, &connection)?;
        }
        Ok(sources)
    })
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
    source: &Source,
    connection: &PgConnection,
) -> QueryResult<Source> {
    diesel::update(sources::table.find(source.id))
        .set(source)
        .get_result(connection)
}

pub fn delete(id: Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(sources::table.find(id)).execute(connection)
    // TODO delete articles under this source?
}
