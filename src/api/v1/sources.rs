use crate::db::{sources::Source, users, users::User, DbConn};
use log;

use serde::{Deserialize, Serialize};

use crate::api::v1::{
    internal_err_resp, ok_resp, user_err_resp, JSONResp, ValidToken,
};

// #[get("/api/v1/sources")]
// pub fn sources_list(conn: DbConn, token: ValidToken) -> JSONResp<Vec<Source>>
// {     let user = users::get(token.username, &conn)?;
//     let user = match users::get(token.username, &conn) {
//         Ok(user) => user,
//         Err(e) => {
//             log::error!("Invalid user for valid token: {}", e);
//             return internal_err_resp("Could not retrieve user");
//         }
//     };
// }
