use crate::{
    api::v1::{
        internal_err_resp, ok_resp, user_err_resp, JSONResp, ValidToken,
    },
    db::{tokens, tokens::Token, users, users::User, DbConn},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use log;
use rocket::{
    http::{Cookie, Cookies},
    State,
};
use rocket_contrib::json::Json;
use time;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::state::Environment;

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
    let passwords_match = verify(login.password.clone(), &user.password)?;
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

    tokens::insert(token, &conn)?;
    ok_resp(ApiTokenResp {
        api_token: api_token,
    })
}

#[post("/api/v1/users/create", data = "<user>")]
pub fn create(conn: DbConn, user: Json<User>) -> JSONResp<String> {
    let hashed_pass = hash(user.password.clone(), DEFAULT_COST)?;
    log::debug!("Hashed {} as {}", user.password.clone(), hashed_pass);

    let user = User {
        username: user.username.clone(),
        password: hashed_pass,
    };

    let username = user.username.clone();
    match users::insert(user, &conn) {
        Ok(_) => ok_resp(format!("Created user {}", username)),
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
    ok_resp(users::get(token.username, &conn)?)
}
