use crate::{
    db::{sources::Source, tags::Tag},
    schema::tagged_sources,
};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(
    Associations, Queryable, AsChangeset, Debug, Identifiable, Insertable,
)]
#[table_name = "tagged_sources"]
#[belongs_to(Tag, foreign_key = "tag")]
#[belongs_to(Source, foreign_key = "source")]
pub struct TaggedSource {
    pub id: Uuid,
    pub tag: Uuid,
    pub source: Uuid,
}

pub fn get(id: Uuid, connection: &PgConnection) -> QueryResult<TaggedSource> {
    tagged_sources::table
        .find(id)
        .get_result::<TaggedSource>(connection)
}

pub fn insert(
    tagged_src: TaggedSource,
    connection: &PgConnection,
) -> QueryResult<TaggedSource> {
    diesel::insert_into(tagged_sources::table)
        .values(tagged_src)
        .get_result(connection)
}

pub fn update(
    tagged_src: TaggedSource,
    connection: &PgConnection,
) -> QueryResult<TaggedSource> {
    diesel::update(tagged_sources::table.find(tagged_src.id.clone()))
        .set(tagged_src)
        .get_result(connection)
}

pub fn delete(id: Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(tagged_sources::table.find(id)).execute(connection)
}
