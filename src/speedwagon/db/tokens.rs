use crate::speedwagon::api::v1::users::Token;
use uuid::Uuid;
use crate::speedwagon::schema::tokens;
use diesel::prelude::*;

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
    diesel::delete(tokens::table.find(id))
        .execute(connection)
}
