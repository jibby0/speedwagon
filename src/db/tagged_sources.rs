
#[derive(Queryable, AsChangeset, Serialize, Deserialize, Debug, Identifiable, Insertable)]
#[table_name = "tagged_sources"]
#[belongs_to(Tag, foreign_key = "tag")]
#[belongs_to(Source, foreign_key = "source")]
pub struct TaggedSource {
    pub id: Uuid,
    pub tag: Uuid,
    pub source: Uuid,
}
