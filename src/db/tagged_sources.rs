use diesel::prelude::*;
use crate::db::sources::Source;
use crate::db::tags::Tag;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Associations, Queryable, AsChangeset, Debug, Identifiable, Insertable)]
#[table_name = "tagged_sources"]
#[belongs_to(Tag, foreign_key = "tag")]
#[belongs_to(Source, foreign_key = "source")]
pub struct TaggedSource {
    pub id: Uuid,
    pub tag: Uuid,
    pub source: Uuid,
}
