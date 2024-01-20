use std::{
    env,
    time::{Duration, SystemTime},
};

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
};
use dotenvy::dotenv;
use rand_core::OsRng;
use reqwest;
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    request::FromRequest,
    response::status::{Created, Custom},
    Request,
};
use rocket::{response::Redirect, serde::json::Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error_handler::{handle_error, CustomError, ErrorInfo},
    models::{AuthenticatedUser, Token, User, UserSensitive},
    DbPool, utils::strings::generate_random_string,
};

#[derive(Deserialize, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub handle: String,
    pub nickname: String,
    pub password: String,
    pub email: String,
}

impl NewUser {
    fn hash_password(&self) -> Result<NewUser, argon2::password_hash::Error> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(self.password.as_bytes(), &salt)?
            .to_string();

        Ok(NewUser {
            handle: self.handle.clone(),
            nickname: self.nickname.clone(),
            password: password_hash,
            email: self.email.clone(),
        })
    }
}

fn verify_password(password: &String, hash: &String) -> Result<(), argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    argon2.verify_password(password.as_bytes(), &PasswordHash::new(&hash)?)
}

fn is_handle_exists(
    handle: &String,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<bool, CustomError> {
    use crate::schema::users;

    let rows = users::table
        .filter(users::handle.eq(handle))
        .execute(conn)
        .map_err(handle_error)?;

    Ok(rows > 0)
}

fn validate_handle(handle: &str) -> Result<(), CustomError> {
    if handle.is_empty() {
        Err(Custom(
            Status::BadRequest,
            Json(ErrorInfo::new("Handle too short".into())),
        ))
    } else {
        Ok(())
    }
}

fn validate_password(password: &str) -> Result<(), CustomError> {
    if password.is_empty() {
        Err(Custom(
            Status::BadRequest,
            Json(ErrorInfo::new("Password too short".into())),
        ))
    } else {
        Ok(())
    }
}

fn validate_nickname(nickname: &str) -> Result<(), CustomError> {
    if nickname.is_empty() {
        Err(Custom(
            Status::BadRequest,
            Json(ErrorInfo::new("Nickname too short".into())),
        ))
    } else {
        Ok(())
    }
}

#[get("/user/check_handle?<handle>")]
pub fn check_handle(handle: String, pool: &rocket::State<DbPool>) -> Result<Value, CustomError> {
    let mut conn = pool.get().expect("Failed to get database connection");

    Ok(json!({ "exists": is_handle_exists(&handle, &mut conn)? }))
}

#[post("/user/register", format = "json", data = "<new_user>")]
pub fn add_user(
    new_user: Json<NewUser>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<User>>, CustomError> {
    use crate::schema::users;

    let mut conn = pool.get().expect("Failed to get database connection");

    validate_handle(&new_user.handle)?;
    validate_password(&new_user.password)?;
    validate_nickname(&new_user.nickname)?;

    if is_handle_exists(&new_user.handle, &mut conn)? {
        return Err(Custom(
            Status::Conflict,
            Json(ErrorInfo::new("Handle already exists".into())),
        ));
    }

    let user: User = diesel::insert_into(users::table)
        .values(new_user.into_inner().hash_password().map_err(|_| {
            Custom(
                Status::InternalServerError,
                Json(ErrorInfo::new("Failed to hash password".to_string())),
            )
        })?)
        .get_result::<UserSensitive>(&mut conn)
        .map_err(handle_error)?
        .into();

    Ok(Created::new(format!("/users/{}", user.handle)).body(Json(user)))
}

#[derive(Deserialize)]
pub struct LoginData {
    pub handle: String,
    pub password: String,
    pub persist: bool,
}

#[derive(Deserialize, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::tokens)]
pub struct InsertToken {
    pub selector: String,
    pub hashed_validator: String,
    pub user_id: i32,
    pub expires: Option<SystemTime>,
}

#[post("/user/login", format = "json", data = "<login_data>")]
pub fn login(
    login_data: Json<LoginData>,
    pool: &rocket::State<DbPool>,
    jar: &CookieJar,
) -> Result<Value, CustomError> {
    use crate::schema::tokens;
    use crate::schema::users;

    dotenv().ok();

    let mut conn = pool.get().expect("Failed to get database connection");

    let user = users::table
        .filter(users::handle.eq(&login_data.handle))
        .select(users::all_columns)
        .first::<UserSensitive>(&mut conn)
        .map_err(|_| {
            Custom(
                Status::Unauthorized,
                Json(ErrorInfo::new("Login failed".to_string())),
            )
        })?;

    verify_password(&login_data.password, &user.password).map_err(|_| {
        Custom(
            Status::Unauthorized,
            Json(ErrorInfo::new("Login failed".to_string())),
        )
    })?;

    let selector = generate_random_string(12);
    let validator = generate_random_string(48);
    let hashed_validator = sha256::digest(&validator);
    let user_id = user.id;
    let token_expires = if login_data.persist {
        None
    } else {
        Some(SystemTime::now() + Duration::from_secs(10800))
    };

    diesel::insert_into(tokens::table)
        .values(InsertToken {
            selector: selector.clone(),
            hashed_validator,
            user_id,
            expires: token_expires,
        })
        .execute(&mut conn)
        .map_err(handle_error)?;

    jar.add(
        Cookie::build(("token", format!("{}:{}", selector, validator)))
            .domain(env::var("DOMAIN").expect("DOMAIN not set"))
            .secure(env::var("SECURE").expect("SECURE not set") == "true")
            .http_only(true)
            .same_site(SameSite::Strict),
    );

    Ok(json!({ "success": true }))
}

#[post("/user/logout")]
pub fn logout(
    user: AuthenticatedUser,
    pool: &rocket::State<DbPool>,
    jar: &CookieJar,
) -> Result<Value, CustomError> {
    use crate::schema::tokens;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::delete(tokens::table)
        .filter(tokens::user_id.eq(user.id))
        .execute(&mut conn)
        .map_err(handle_error)?;

    jar.remove(
        Cookie::build("token")
            .domain("neon.nuty.works")
            .secure(true)
            .http_only(true)
            .same_site(SameSite::Strict),
    );

    Ok(json!({ "success": true }))
}

#[derive(Debug)]
pub enum AuthorizationError {
    DatabaseConnection,
    TokenMissing,
    TokenInvalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = AuthorizationError;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        use crate::schema::tokens;
        use crate::schema::user_circles;
        use crate::schema::users;

        dotenv().ok();

        let pool = request.guard::<&rocket::State<DbPool>>().await;
        let mut conn = match pool.succeeded() {
            Some(pool) => pool.get().expect("Failed to get database connection"),
            None => {
                return rocket::request::Outcome::Error((
                    Status::InternalServerError,
                    AuthorizationError::DatabaseConnection,
                ))
            }
        };

        let token = match request.cookies().get("token") {
            Some(token) => token.value().to_string(),
            None => {
                return rocket::request::Outcome::Error((
                    Status::Unauthorized,
                    AuthorizationError::TokenMissing,
                ))
            }
        };

        let (selector, validator) =
            if let [selector, validator] = token.split(":").collect::<Vec<_>>()[..] {
                (selector, validator)
            } else {
                return rocket::request::Outcome::Error((
                    Status::Unauthorized,
                    AuthorizationError::TokenInvalid,
                ));
            };
        let hashed_validator = sha256::digest(validator);

        let token: Token = match tokens::table
            .filter(tokens::selector.eq(selector))
            .select(tokens::all_columns)
            .first::<Token>(&mut conn)
        {
            Ok(token) => token,
            Err(_) => {
                return rocket::request::Outcome::Error((
                    Status::Unauthorized,
                    AuthorizationError::TokenInvalid,
                ))
            }
        };

        if let Some(expires) = token.expires {
            if expires < SystemTime::now() {
                return rocket::request::Outcome::Error((
                    Status::Unauthorized,
                    AuthorizationError::TokenInvalid,
                ));
            }
        }

        if token.hashed_validator != hashed_validator {
            return rocket::request::Outcome::Error((
                Status::Unauthorized,
                AuthorizationError::TokenInvalid,
            ));
        }

        let user = match users::table
            .filter(users::id.eq(token.user_id))
            .select(users::all_columns)
            .first::<UserSensitive>(&mut conn)
        {
            Ok(user) => user,
            Err(_) => {
                return rocket::request::Outcome::Error((
                    Status::InternalServerError,
                    AuthorizationError::DatabaseConnection,
                ))
            }
        };

        let user_circles = match user_circles::table
            .filter(user_circles::user_id.eq(user.id))
            .select(user_circles::circle_id)
            .load::<i32>(&mut conn)
        {
            Ok(circles) => circles,
            Err(_) => {
                return rocket::request::Outcome::Error((
                    Status::InternalServerError,
                    AuthorizationError::DatabaseConnection,
                ))
            }
        };

        rocket::request::Outcome::Success(AuthenticatedUser {
            id: user.id,
            handle: user.handle,
            nickname: user.nickname,
            twitter_id: user.twitter_id,
            email: user.email,
            role: user.role,
            circles: user_circles,
        })
    }
}

#[derive(Deserialize)]
pub struct UpdateUser {
    nickname: Option<String>,
    email: Option<String>,
    password: String,
    new_password: Option<String>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUserHashed {
    nickname: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

impl UpdateUser {
    fn hash_password(&self) -> Result<UpdateUserHashed, argon2::password_hash::Error> {
        if let Some(new_password) = &self.new_password {
            let argon2 = Argon2::default();
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = argon2
                .hash_password(new_password.as_bytes(), &salt)?
                .to_string();

            Ok(UpdateUserHashed {
                nickname: self.nickname.clone(),
                password: Some(password_hash),
                email: self.email.clone(),
            })
        } else {
            Ok(UpdateUserHashed {
                nickname: self.nickname.clone(),
                password: None,
                email: self.email.clone(),
            })
        }
    }
}

#[patch("/users/me", format = "json", data = "<update_user>")]
pub fn patch_me(
    user: AuthenticatedUser,
    update_user: Json<UpdateUser>,
    pool: &rocket::State<DbPool>,
    jar: &CookieJar,
) -> Result<Json<User>, CustomError> {
    use crate::schema::tokens;
    use crate::schema::users;

    dotenv().ok();

    let mut conn = pool.get().expect("Failed to get database connection");

    if let Some(new_password) = &update_user.new_password {
        validate_password(new_password)?;
    }

    if let Some(nickname) = &update_user.nickname {
        validate_nickname(nickname)?;
    }

    let user = users::table
        .filter(users::handle.eq(&user.handle))
        .select(users::all_columns)
        .first::<UserSensitive>(&mut conn)
        .map_err(|_| {
            Custom(
                Status::Unauthorized,
                Json(ErrorInfo::new("Login failed".to_string())),
            )
        })?;

    verify_password(&update_user.password, &user.password).map_err(|_| {
        Custom(
            Status::Unauthorized,
            Json(ErrorInfo::new("Login failed".to_string())),
        )
    })?;

    if let Some(_) = update_user.new_password {
        diesel::delete(tokens::table)
            .filter(tokens::user_id.eq(user.id))
            .execute(&mut conn)
            .map_err(handle_error)?;

        jar.remove(
            Cookie::build("token")
                .domain(env::var("DOMAIN").expect("DOMAIN not set"))
                .secure(env::var("SECURE").expect("SECURE not set") == "true")
                .http_only(true)
                .same_site(SameSite::Strict),
        );
    }

    Ok(Json(
        diesel::update(users::table)
            .set(update_user.into_inner().hash_password().map_err(|_| {
                Custom(
                    Status::InternalServerError,
                    Json(ErrorInfo::new("Failed to hash password".to_string())),
                )
            })?)
            .get_result::<UserSensitive>(&mut conn)
            .map_err(handle_error)?
            .into(),
    ))
}

#[delete("/users/me")]
pub fn delete_me(
    user: AuthenticatedUser,
    pool: &rocket::State<DbPool>,
) -> Result<Value, CustomError> {
    use crate::schema::users;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::delete(users::table)
        .filter(users::id.eq(user.id))
        .execute(&mut conn)
        .map_err(handle_error)?;

    Ok(json!({ "success": true }))
}

#[get("/users/me")]
pub fn get_me(user: AuthenticatedUser) -> Result<Json<AuthenticatedUser>, CustomError> {
    Ok(Json(user))
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
struct NewCodeVerifier {
    pub oauth_state: String,
    pub code_verifier: String,
}

#[get("/oauth/twitter/new")]
pub fn new_twitter_oauth(
    user: AuthenticatedUser,
    pool: &rocket::State<DbPool>,
) -> Result<Redirect, CustomError> {
    use crate::schema::users;

    dotenv().ok();

    let oauth_state = generate_random_string(16);
    let code_verifier = generate_random_string(128);
    let code_challenge =
        URL_SAFE_NO_PAD.encode(hex::decode(sha256::digest(&code_verifier)).unwrap());

    let mut conn = pool.get().expect("Failed to get database connection");

    let url = format!(
        "https://twitter.com/i/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}%2Fapi%2Foauth%2Ftwitter&scope=tweet.read%20users.read&state={}&code_challenge={}&code_challenge_method=S256",
        env::var("CLIENT_ID").expect("CLIENT_ID not set"),
        env::var("BASE_URL").expect("BASE_URL not set").replace(":", "%3A").replace("/", "%2F"),
        oauth_state,
        code_challenge
    );

    diesel::update(users::table)
        .filter(users::id.eq(user.id))
        .set(NewCodeVerifier { oauth_state, code_verifier })
        .execute(&mut conn)
        .map_err(handle_error)?;

    Ok(Redirect::temporary(url))
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
struct NewTwitterId {
    pub twitter_id: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::user_circles)]
struct NewUserCircle {
    pub user_id: i32,
    pub circle_id: i32,
}

#[get("/oauth/twitter?<code>&<state>")]
pub async fn check_twitter_oauth(
    state: String,
    code: String,
    pool: &rocket::State<DbPool>,
) -> Result<Redirect, CustomError> {
    use crate::schema::users;
    use crate::schema::user_circles;
    use crate::schema::circle_artists;
    use crate::schema::artists;

    dotenv().ok();
    let base_url = env::var("BASE_URL").expect("BASE_URL not set");
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not set");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set");

    let mut conn = pool.get().expect("Failed to get database connection");

    let verifier = users::table
        .filter(users::oauth_state.eq(&state))
        .select(users::code_verifier)
        .first::<Option<String>>(&mut conn)
        .map_err(handle_error)?;

    if let Some(verifier) = verifier {
        let client = reqwest::Client::new();
        let response = client.post("https://api.twitter.com/2/oauth2/token")
            .basic_auth(client_id, Some(client_secret))
            .form(&[
                ("code", code.as_str()),
                ("grant_type", "authorization_code"),
                (
                    "redirect_uri", 
                    format!("{}/api/oauth/twitter", base_url).as_str(),
                ),
                ("code_verifier", verifier.as_str()),
            ])
            .send()
            .await
            .map_err(|_| Custom(Status::InternalServerError, Json(ErrorInfo::new("Twitter did not respond".into()))))?
            .json::<Value>()
            .await
            .map_err(|_| Custom(Status::InternalServerError, Json(ErrorInfo::new("internal_server_error".into()))))?;

        let access_token = response["access_token"].as_str().unwrap();

        let response = client.get("https://api.twitter.com/2/users/me")
            .bearer_auth(access_token.to_string().trim_matches('"'))
            .send()
            .await
            .map_err(|_| Custom(Status::InternalServerError, Json(ErrorInfo::new("Twitter did not respond".into()))))?
            .json::<Value>()
            .await
            .map_err(|_| Custom(Status::InternalServerError, Json(ErrorInfo::new("internal_server_error".into()))))?;

        let twitter_id = response["data"]["username"].as_str().unwrap().to_string();

        let participating_circles = artists::table
            .filter(artists::account_url.eq_any(vec![
                format!("https://twitter.com/{}", twitter_id),
                format!("https://twitter.com/{}/", twitter_id),
                format!("https://x.com/{}", twitter_id),
                format!("https://x.com/{}/", twitter_id),
            ]))
            .inner_join(circle_artists::table.on(artists::id.eq(circle_artists::artist_id)))
            .select(circle_artists::circle_id)
            .load::<i32>(&mut conn)
            .map_err(handle_error)?;

        let user_id = users::table.filter(users::oauth_state.eq(&state))
            .select(users::id)
            .first::<i32>(&mut conn)
            .map_err(handle_error)?;

        for circle_id in participating_circles {
            diesel::insert_into(user_circles::table)
                .values(NewUserCircle { user_id, circle_id })
                .execute(&mut conn)
                .map_err(handle_error)?;
        }

        diesel::update(users::table)
            .filter(users::oauth_state.eq(&state))
            .set(NewTwitterId { twitter_id })
            .execute(&mut conn)
            .map_err(handle_error)?;
    } else {
        return Err(Custom(Status::BadRequest, Json(ErrorInfo::new("Invalid request".into()))));
    }

    Ok(Redirect::temporary("/profile"))
}