use crate::db::users::User;
use crate::schema::tokens;
use diesel::prelude::*;
use time;
use uuid::Uuid;

pub type TokenId = Uuid;

#[derive(Queryable, AsChangeset, Debug, Associations, Insertable)]
#[table_name = "tokens"]
#[belongs_to(User, foreign_key = "username")]
pub struct Token {
    pub id: TokenId,
    pub username: String,
    pub expires: time::Timespec,
}

pub fn get(id: Uuid, connection: &PgConnection) -> QueryResult<Token> {
    tokens::table.find(id).get_result::<Token>(connection)
}

pub fn insert(token: Token, connection: &PgConnection) -> QueryResult<Token> {
    diesel::insert_into(tokens::table)
        .values(token)
        .get_result(connection)
}

pub fn update(token: Token, connection: &PgConnection) -> QueryResult<Token> {
    diesel::update(tokens::table.find(token.id))
        .set(&token)
        .get_result(connection)
}

pub fn delete(id: Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(tokens::table.find(id)).execute(connection)
}
