use crate::db::{sources, sources::Source, users, DbConn};

use crate::api::v1::{ok_resp, JSONResp, ValidToken};

#[get("/api/v1/sources")]
pub fn sources_list(conn: DbConn, token: ValidToken) -> JSONResp<Vec<Source>> {
    let user = users::get(token.username, &conn)?;
    let sources = sources::all_from_user(user.username, &conn)?;
    ok_resp(sources)
}
