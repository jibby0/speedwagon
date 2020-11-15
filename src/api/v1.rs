pub mod items;
pub mod sources;
pub mod users;

use crate::db::{tokens, DbConn, Pool};
use bcrypt::BcryptError;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    response::{status::Custom, Responder, Response},
    State,
};
use rocket_contrib::json::Json;
use std::{error::Error, result::Result};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    status: String,
    contents: T,
}

#[derive(Debug)]
pub struct ApiError(Custom<Json<Resp<String>>>);
impl ApiError {
    fn new(status: Status, contents: String) -> ApiError {
        ApiError(Custom(
            status,
            Json(Resp {
                status: "error".into(),
                contents,
            }),
        ))
    }

    fn internal(error: &dyn Error) -> ApiError {
        log::error!("{}", error);
        ApiError::new(
            Status::InternalServerError,
            String::from("Internal error"),
        )
    }
}

/// Allow error handling with `?` for 500 errors.
impl From<diesel::result::Error> for ApiError {
    fn from(error: diesel::result::Error) -> Self {
        ApiError::internal(&error)
    }
}

impl From<BcryptError> for ApiError {
    fn from(error: BcryptError) -> Self {
        ApiError::internal(&error)
    }
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        self.0.respond_to(req)
    }
}

pub type JSONResp<T> = Result<Json<Resp<T>>, ApiError>;

pub fn ok_resp<T: Serialize>(x: T) -> JSONResp<T> {
    Ok(Json(Resp {
        status: "ok".into(),
        contents: x,
    }))
}

pub fn user_err_resp<U: Into<String>, T>(x: U) -> JSONResp<T> {
    Err(ApiError::new(Status::BadRequest, x.into()))
}

pub fn internal_err_resp<U: Into<String>, T>(x: U) -> JSONResp<T> {
    Err(ApiError::new(Status::InternalServerError, x.into()))
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
                    [_, token] => Uuid::parse_str(token).ok().map(|token| {
                        log::debug!("Found header token {}", token);
                        token
                    }),
                    _ => None,
                }
            })
            .or_else(|| {
                request
                    .cookies()
                    .get_private("api_token")
                    .and_then(|cookie| cookie.value().parse().ok())
                    .map(|token| {
                        log::debug!("Found cookie token {}", token);
                        token
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
