use crate::{
    api::v1::{
        internal_err_resp, ok_resp, user_err_resp, JSONResp, ValidToken,
    },
    db::{tokens, tokens::Token, users, users::User, DbConn},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::{
    http::{Cookie, Cookies},
    State,
};
use rocket_contrib::json::Json;
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

#[post("/user/login", data = "<login>")]
pub fn user_login(
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
    ok_resp(ApiTokenResp { api_token })
}

#[post("/user", data = "<user>")]
pub fn user_create(
    conn: DbConn,
    user: Json<User>,
    rocket_env: State<Environment>,
) -> JSONResp<String> {
    let hashed_pass = hash(user.password.clone(), DEFAULT_COST)?;
    if !rocket_env.inner().0.is_prod() {
        log::debug!("Hashed {} as {}", user.password.clone(), hashed_pass);
    }

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

#[put("/user", data = "<user>")]
pub fn user_change_pass(
    conn: DbConn,
    user: Json<User>,
    rocket_env: State<Environment>,
    token: ValidToken,
) -> JSONResp<String> {
    if token.username != user.username {
        return user_err_resp(format!(
            "Signed in as user {}, cannot change password for user {}",
            token.username, user.username
        ));
    }

    let hashed_pass = hash(user.password.clone(), DEFAULT_COST)?;
    if !rocket_env.inner().0.is_prod() {
        log::debug!("Hashed {} as {}", user.password.clone(), hashed_pass);
    }

    let user = User {
        username: user.username.clone(),
        password: hashed_pass,
    };

    let username = user.username.clone();
    users::update(user, &conn)?;

    ok_resp(format!("Created user {}", username))
}

#[post("/user/logout")]
pub fn user_logout(
    conn: DbConn,
    token: ValidToken,
    mut cookies: Cookies<'_>,
) -> JSONResp<&'static str> {
    cookies.remove_private(Cookie::named("api_token"));
    match tokens::delete(token.id, &conn) {
        Ok(_) => ok_resp("Successfully logged out"),
        Err(e) => {
            log::error!("Error removing valid DB token: {}", e);
            internal_err_resp("Could not log user out")
        }
    }
}

#[delete("/user")]
pub fn user_delete(
    conn: DbConn,
    token: ValidToken,
    mut cookies: Cookies<'_>,
) -> JSONResp<&'static str> {
    cookies.remove_private(Cookie::named("api_token"));

    let username = token.username;
    for t in tokens::all_for_user(username.clone(), &conn)? {
        tokens::delete(t.id, &conn)?;
    }
    users::delete(username, &conn)?;

    ok_resp("Successfully deleted user")
}

#[get("/user")]
pub fn user_index(conn: DbConn, token: ValidToken) -> JSONResp<User> {
    ok_resp(users::get(token.username, &conn)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api::v1::Resp, setup_rocket::setup_rocket};
    use rocket::{
        http::{ContentType, Header, Status},
        local::Client,
    };

    #[test]
    fn api_user_lifecycle() {
        // Create
        let client =
            Client::new(setup_rocket()).expect("valid rocket instance");

        let payload = serde_json::to_value(users::User {
            username: "foo".into(),
            password: "bar".into(),
        })
        .unwrap()
        .to_string();

        let mut response = client
            .post("/api/v1/user")
            .body(payload)
            .header(ContentType::JSON)
            .dispatch();
        let resp_obj: Resp<String> =
            serde_json::from_str(&response.body_string().unwrap()).unwrap();

        assert_eq!(resp_obj.contents, "Created user foo".to_string());
        assert_eq!(resp_obj.status, "ok");
        assert_eq!(response.status(), Status::Ok);

        // Login
        let payload = serde_json::to_value(UserLogin {
            username: "foo".into(),
            password: "bar".into(),
            persistent: false,
        })
        .unwrap()
        .to_string();

        let mut response = client
            .post("/api/v1/user/login")
            .body(payload)
            .header(ContentType::JSON)
            .dispatch();
        let resp_obj: Resp<ApiTokenResp> =
            serde_json::from_str(&response.body_string().unwrap()).unwrap();

        assert_eq!(resp_obj.status, "ok");
        assert_eq!(response.status(), Status::Ok);

        let api_token = resp_obj.contents.api_token;

        // Delete
        let mut response = client
            .delete("/api/v1/user")
            .header(Header::new(
                "Authorization",
                format!("Bearer {}", api_token),
            ))
            .dispatch();

        let resp_obj: Resp<String> =
            serde_json::from_str(&response.body_string().unwrap()).unwrap();
        assert_eq!(resp_obj.status, "ok");
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(resp_obj.contents, "Successfully deleted user".to_string());
    }
}
