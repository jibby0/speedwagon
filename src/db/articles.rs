use crate::schema::users;
use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use time;
use uuid::Uuid;

#[derive(Queryable, AsChangeset, Serialize, Deserialize, Debug, Identifiable, Insertable)]
#[table_name = "articles"]
#[belongs_to(Source, foreign_key = "source")]
pub struct Article {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub date: time::Timespec,
    pub link: String,
    pub author: String,
    pub source: Uuid,
    // TODO more
    // icon/thumbnail?
}

pub fn all_from_source(source: Uuid, connection: &PgConnection) -> QueryResult<Vec<Article>> {
    articles::table.filter(source.eq(source)).load::<Article>(&*connection)
}

pub fn get(id: Uuid, connection: &PgConnection) -> QueryResult<Article> {
    articles::table.find(id).get_result::<Article>(connection)
}

pub fn insert(article: Article, connection: &PgConnection) -> QueryResult<Article> {
    diesel::insert_into(articles::table)
        .values(article)
        .get_result(connection)
}

pub fn update(article: Article, connection: &PgConnection) -> QueryResult<Article> {
    diesel::update(articles::table.find(article.id.clone()))
        .set(article)
        .get_result(connection)
}

pub fn delete(id: Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(articles::table.find(id))
        .execute(connection)
}
