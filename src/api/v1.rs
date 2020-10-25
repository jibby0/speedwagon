pub mod items;
pub mod sources;
pub mod users;

use crate::db::{tokens, DbConn, Pool};
use log;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    response::{status::Custom, Responder, Response},
    State,
};
use rocket_contrib::json::Json;
use std::{error::Error, result::Result};
use time;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    status: &'static str,
    contents: T,
}

struct MyCustom<R>(Custom<R>);

// TODO implement `From` for JSONResp
// `?` would be useful in many API methods, just logging at
// ERROR & returning a generic 500 "Internal error" message.
pub type JSONResp<T> = Result<Json<Resp<T>>, Custom<Json<Resp<String>>>>;

// impl<'r, T: Responder<'r>> Responder<'r> for JSONResp<T> {
//     fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
//         match self {
//             Ok(t) => t.respond_to(req),
//             Err(e) => e.0.respond_to(req)
//         }
//     }
// }

// impl From<Box<dyn Error>> for MyCustom<Json<Resp<String>>> {
//     fn from(error: Box<dyn Error>) -> Self {
//        MyCustom(Custom(
//         Status::InternalServerError,
//         Json(Resp {
//             status: "error",
//             contents: String::from("Internal error")
//         })))
//     }
// }

pub fn ok_resp<T: Serialize>(x: T) -> JSONResp<T> {
    Ok(Json(Resp {
        status: "ok",
        contents: x,
    }))
}

pub fn user_err_resp<U: Into<String>, T>(x: U) -> JSONResp<T> {
    Err(Custom(
        Status::BadRequest,
        Json(Resp {
            status: "error",
            contents: x.into(),
        }),
    ))
}

pub fn internal_err_resp<U: Into<String>, T>(x: U) -> JSONResp<T> {
    Err(Custom(
        Status::InternalServerError,
        Json(Resp {
            status: "error",
            contents: x.into(),
        }),
    ))
}

pub struct ValidToken {
    pub id: tokens::TokenId,
    pub username: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for ValidToken {
    type Error = ();

    fn from_request(
        request: &'a Request<'r>,
    ) -> Outcome<ValidToken, Self::Error> {
        let opt_token = request
            .headers()
            .get_one("Authorization")
            .and_then(|bearer| {
                // This is kinda awful, but whatever
                let split_bearer: Vec<&str> =
                    bearer.split_ascii_whitespace().collect();
                match split_bearer[..] {
                    [_, token] => {
                        Uuid::parse_str(token).ok().and_then(|token| {
                            log::debug!("Found header token {}", token);
                            Some(token)
                        })
                    }
                    _ => None,
                }
            })
            .or_else(|| {
                request
                    .cookies()
                    .get_private("api_token")
                    .and_then(|cookie| cookie.value().parse().ok())
                    .and_then(|token| {
                        log::debug!("Found cookie token {}", token);
                        Some(token)
                    })
            });
        let token_id = match opt_token {
            Some(token) => token,
            None => return Outcome::Forward(()),
        };

        let pool = request.guard::<State<Pool>>()?;
        let conn = match pool.get() {
            Ok(conn) => DbConn(conn),
            Err(_) => {
                return Outcome::Failure((Status::ServiceUnavailable, ()))
            }
        };

        let token = match tokens::get(token_id, &conn) {
            Ok(token) => token,
            Err(_) => return Outcome::Failure((Status::Unauthorized, ())),
        };
        if token.expires < time::now().to_timespec() {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        Outcome::Success(ValidToken {
            id: token.id,
            username: token.username,
        })
    }
}
