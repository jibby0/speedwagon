use rocket::response::status::Custom;
use rocket::http::Status;
use rocket::http::{Cookie, Cookies};
use rocket_contrib::json::Json;
use uuid::Uuid;
use bcrypt::{DEFAULT_COST, hash, verify};
use time::Tm;
use log;

use serde::{Serialize, Serializer, Deserialize};

use crate::speedwagon::db::DbConn;
use crate::speedwagon::db::users as db_users;
use crate::speedwagon::schema::users;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    status: &'static str,
    contents: T,
}
pub type JSONResp<T> = Result<Json<Resp<T>>, Custom<Json<Resp<String>>>>;

pub fn ok_resp<T: Serialize>(x: T) -> JSONResp<T> {
    Ok(Json(Resp{
        status: "ok",
        contents: x,
    }))
}

pub fn err_resp<U: Into<String>, T>(x: U) -> JSONResp<T> {
    Err(
        Custom(Status::BadRequest,
        Json(Resp{
            status: "error",
            contents: x.into(),
        })
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTokenResp {
    api_token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    username: String,
    password: String,
    persistent: bool
}

#[derive(Queryable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name = "users"]
pub struct User {
    pub username: String,
    pub password: String
}

pub struct SerdeTm(Tm);

impl Serialize for SerdeTm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let SerdeTm(tm) = &self;
        serializer.collect_str(&tm.rfc3339())
    }
}

impl Deserialize for SerdeTm {
    fn deserialize<D>(&self, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let SerializableTm(tm) = &self;
        serializer.collect_str(&tm.rfc3339())
    }
}


#[derive(Queryable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name = "tokens"]
pub struct Token {
    pub id: Uuid,
    pub user: User,
    pub expires: SerializableTm
}

// TODO make this work for DB users
// impl<'a, 'r> FromRequest<'a, 'r> for User {
//    type Error = std::convert::Infallible;
//
//    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, Self::Error> {
//        request.cookies()
//            .get_private("user_cookie")
//            .and_then(|cookie| cookie.value().parse().ok())
//            .map(|id| User(id))
//            .or_forward(())
//    }
//}

#[post("/api/v1/users/login", data = "<login>")]
pub fn login(mut cookies: Cookies<'_>, connection: DbConn, login: Json<UserLogin>) -> JSONResp<ApiTokenResp> {
    let user = match db_users::get(login.username, &connection) {
        Err(e) => return err_resp(format!("User {} not found", login.username)),
        Ok(u) => u
    };
    log::debug!("{:?}", user);
    let passwords_match = match verify(login.password.clone(), &user.password) {
        Err(e) => {
            log::error!("Error verifying password: {}", e);
            return err_resp("Could not verify password")
        }
        Ok(p) => p
    };
    if passwords_match {
        let api_token = Uuid::new_v4().to_string();
        let mut cookie = Cookie::new("api_token", api_token.clone());
        cookie.set_secure(true);
        if login.persistent {
            cookie.make_permanent();
        }
        cookies.add_private(cookie);

        ok_resp(ApiTokenResp{api_token: api_token})
    } else {
        err_resp("Invalid username/password.")
    }
}

#[post("/api/v1/users/create", data = "<login>")]
pub fn create(login: Json<User>) -> JSONResp<String> {
    let hashed_pass = match hash(login.password.clone(), DEFAULT_COST) {
        Err(e) => {
            log::error!("Error hashing password: {}", e);
            return err_resp("Could not create user")
        }
        Ok(p) => p
    };
    log::debug!("Hashed {} as {}", login.password, hashed_pass);
    ok_resp(format!("Created user {}", login.username))
}

#[post("/api/v1/users/logout")]
pub fn logout(mut cookies: Cookies<'_>) -> JSONResp<&'static str> {
    cookies.remove_private(Cookie::named("user_cookie"));
    ok_resp("Successfully logged out")
}

#[get("/api/v1/index")]
pub fn user_index(user: User) -> JSONResp<User> {
    ok_resp(user)
}
