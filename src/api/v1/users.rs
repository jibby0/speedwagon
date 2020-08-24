use crate::db::{tokens, tokens::Token, users, users::User, DbConn, Pool};
use bcrypt::{hash, verify, DEFAULT_COST};
use log;
use rocket::{
    http::{Cookie, Cookies, Status},
    request::{FromRequest, Outcome, Request},
    response::status::Custom,
    State,
};
use rocket_contrib::json::Json;
use time;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::state::Environment;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    status: &'static str,
    contents: T,
}
pub type JSONResp<T> = Result<Json<Resp<T>>, Custom<Json<Resp<String>>>>;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTokenResp {
    api_token: tokens::TokenId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    username: String,
    password: String,
    persistent: bool,
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

#[post("/api/v1/users/login", data = "<login>")]
pub fn login(
    mut cookies: Cookies<'_>,
    conn: DbConn,
    rocket_env: State<Environment>,
    login: Json<UserLogin>,
) -> JSONResp<ApiTokenResp> {
    let user = match users::get(login.username.clone(), &conn) {
        Err(_) => {
            return user_err_resp(format!("User {} not found", login.username))
        }
        Ok(u) => u,
    };
    let passwords_match = match verify(login.password.clone(), &user.password) {
        Err(e) => {
            log::error!("Error verifying password: {}", e);
            return internal_err_resp("Could not verify password");
        }
        Ok(p) => p,
    };
    if !passwords_match {
        return user_err_resp("Invalid username/password.");
    }

    let api_token = Uuid::new_v4();

    let expiration = time::now()
        + if login.persistent {
            time::Duration::days(365 * 20)
        } else {
            time::Duration::days(1)
        };
    let token = Token {
        id: api_token,
        username: user.username,
        expires: expiration.to_timespec(),
    };
    let mut cookie = Cookie::new("api_token", api_token.to_string());
    cookie.set_secure(rocket_env.inner().0.is_prod());
    cookie.set_expires(expiration);
    cookies.add_private(cookie);

    match tokens::insert(token, &conn) {
        Err(e) => {
            log::error!("Error inserting token into DB: {}", e);
            internal_err_resp("Could not save user token")
        }
        Ok(_) => ok_resp(ApiTokenResp {
            api_token: api_token,
        }),
    }
}

#[post("/api/v1/users/create", data = "<login>")]
pub fn create(conn: DbConn, login: Json<User>) -> JSONResp<String> {
    let hashed_pass = match hash(login.password.clone(), DEFAULT_COST) {
        Err(e) => {
            log::error!("Error hashing password: {}", e);
            return internal_err_resp("Could not hash password");
        }
        Ok(p) => p,
    };
    log::debug!("Hashed {} as {}", login.password, hashed_pass);

    let user = User {
        username: login.username.clone(),
        password: hashed_pass,
    };
    match users::insert(user, &conn) {
        Ok(_) => ok_resp(format!("Created user {}", login.username)),
        Err(e) => user_err_resp(format!("Could not create user: {}", e)),
    }
}

#[post("/api/v1/users/logout")]
pub fn logout(
    conn: DbConn,
    token: ValidToken,
    mut cookies: Cookies<'_>,
) -> JSONResp<&'static str> {
    cookies.remove_private(Cookie::named("api_token"));
    match tokens::delete(token.id, &conn) {
        Ok(_) => ok_resp("Successfully logged out"),
        Err(e) => {
            log::error!("Error removing valid DB token: {}", e);
            return internal_err_resp("Could not log user out");
        }
    }
}

#[get("/api/v1/index")]
pub fn user_index(conn: DbConn, token: ValidToken) -> JSONResp<User> {
    match users::get(token.username, &conn) {
        Ok(user) => ok_resp(user),
        Err(e) => {
            log::error!("Invalid user for valid token: {}", e);
            return internal_err_resp("Could not retrieve user");
        }
    }
}
