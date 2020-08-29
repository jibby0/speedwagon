use crate::{db::users::User, schema::tags};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(
    Associations, Queryable, AsChangeset, Debug, Identifiable, Insertable,
)]
#[table_name = "tags"]
#[belongs_to(User, foreign_key = "owner")]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    /* TODO more
     * color? */
}

pub fn all_from_user(
    username: String,
    connection: &PgConnection,
) -> QueryResult<Vec<Tag>> {
    tags::table
        .filter(tags::owner.eq(username))
        .load::<Tag>(&*connection)
}

pub fn get(id: Uuid, connection: &PgConnection) -> QueryResult<Tag> {
    tags::table.find(id).get_result::<Tag>(connection)
}

pub fn insert(tag: Tag, connection: &PgConnection) -> QueryResult<Tag> {
    diesel::insert_into(tags::table)
        .values(tag)
        .get_result(connection)
}

pub fn update(tag: Tag, connection: &PgConnection) -> QueryResult<Tag> {
    diesel::update(tags::table.find(tag.id.clone()))
        .set(tag)
        .get_result(connection)
}

pub fn delete(id: Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(tags::table.find(id)).execute(connection)
}
