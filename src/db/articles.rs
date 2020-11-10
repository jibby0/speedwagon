use crate::{db::sources::Source, schema::articles};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(
    Associations, Queryable, AsChangeset, Debug, Identifiable, Insertable,
)]
#[table_name = "articles"]
#[belongs_to(Source, foreign_key = "source")]
pub struct Article {
    pub id: Uuid,
    pub title: Option<String>,
    pub published: Option<time::Timespec>,
    pub source_info: serde_json::Value,
    pub summary: Option<String>,
    pub content: serde_json::Value,
    pub rights: Option<String>,
    pub links: serde_json::Value,
    pub authors: serde_json::Value,
    pub categories: serde_json::Value,
    pub comments_url: Option<String>,
    pub extensions: serde_json::Value,
    pub source: Uuid,
    pub id_from_source: Option<String>,
    /* TODO more
     * icon/thumbnail? */
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleSource {
    pub title: Option<String>,
    pub links: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleLink {
    pub url: Option<String>,
    pub relationship: Option<String>,
    pub title: Option<String>,
}

pub fn all_from_source(
    source: Uuid,
    connection: &PgConnection,
) -> QueryResult<Vec<Article>> {
    articles::table
        .filter(articles::source.eq(&source))
        .load::<Article>(&*connection)
}

pub fn get(id: Uuid, connection: &PgConnection) -> QueryResult<Article> {
    articles::table.find(id).get_result::<Article>(connection)
}

pub fn insert(
    article: Article,
    connection: &PgConnection,
) -> QueryResult<Article> {
    diesel::insert_into(articles::table)
        .values(article)
        .get_result(connection)
}

pub fn update(
    article: Article,
    connection: &PgConnection,
) -> QueryResult<Article> {
    diesel::update(articles::table.find(article.id))
        .set(article)
        .get_result(connection)
}

pub fn delete(id: Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(articles::table.find(id)).execute(connection)
}
