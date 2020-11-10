use crate::{
    api::v1::{ok_resp, user_err_resp, JSONResp, ValidToken},
    db::{
        sources::{self, Source, SourceData},
        users, DbConn,
    },
};

use rocket_contrib::{self, json::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceCreatePayload {
    pub title: Option<String>,
    pub source_data: SourceData,
    pub post_filter: String,
}

/// FromData is not implemented on rocket_contrib's UUID, so
/// this JSON payload is used
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceIDPayload {
    pub id: Uuid,
}

#[get("/source")]
pub fn sources_list(conn: DbConn, token: ValidToken) -> JSONResp<Vec<Source>> {
    let user = users::get(token.username, &conn)?;
    let sources = sources::all_from_user(user.username, &conn)?;
    ok_resp(sources)
}

#[put("/source", data = "<source>")]
pub fn source_update(
    conn: DbConn,
    token: ValidToken,
    source: Json<Source>,
) -> JSONResp<Source> {
    let user = users::get(token.username, &conn)?;
    let old_source = sources::get(source.id, &conn)?;
    if old_source.creator != user.username {
        return user_err_resp(format!(
            "Unauthorized to update source {}",
            source.id
        ));
    }
    let updated_source = sources::update(&source.into_inner(), &conn)?;
    ok_resp(updated_source)
}

#[post("/source", data = "<source>")]
pub fn source_create(
    conn: DbConn,
    token: ValidToken,
    source: Json<SourceCreatePayload>,
) -> JSONResp<Source> {
    let s = source.into_inner();
    let new_source = sources::insert(
        Source::new(
            None,
            s.title.unwrap_or_else(|| "".to_string()),
            serde_json::to_value(s.source_data).unwrap(),
            s.post_filter,
            token.username,
        ),
        &conn,
    )?;
    ok_resp(new_source)
}

#[delete("/source", data = "<source>")]
pub fn source_delete(
    conn: DbConn,
    token: ValidToken,
    source: Json<SourceIDPayload>,
) -> JSONResp<String> {
    let user = users::get(token.username, &conn)?;
    let source_to_delete = sources::get(source.into_inner().id, &conn)?;
    if source_to_delete.creator != user.username {
        return user_err_resp(format!(
            "Unauthorized to delete source {}",
            source_to_delete.id
        ));
    }
    sources::delete(source_to_delete.id, &conn)?;
    ok_resp(format!(
        "Successfully deleted source {}",
        source_to_delete.id
    ))
}
