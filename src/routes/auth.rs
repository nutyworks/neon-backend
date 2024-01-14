use std::{
    env,
    time::{Duration, SystemTime},
};

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, ManageConnection, PooledConnection},
};
use dotenvy::dotenv;
use rand_core::{OsRng, RngCore};
use rocket::serde::json::Json;
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    request::FromRequest,
    response::status::{Created, Custom},
    Request,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error_handler::{handle_error, CustomError, ErrorInfo},
    models::{AuthenticatedUser, Token, User, UserSensitive},
    DbPool,
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

fn generate_random_string(len: usize) -> String {
    vec![0; len]
        .iter()
        .map(|_| {
            let v = rand_core::OsRng.next_u32() % 62;
            if v <= 9 {
                ('0' as u32) + v
            } else if 10 <= v && v <= 35 {
                ('A' as u32) + v - 10
            } else {
                ('a' as u32) + v - 36
            }
        })
        .map(char::from_u32)
        .scan(Some(""), |acc, x| match acc {
            Some(acc) => match x {
                Some(x) => Some(acc.to_string() + &x.to_string()),
                None => None,
            },
            None => None,
        })
        .collect()
}

fn validate_handle(handle: &str) -> Result<(), CustomError> {
    if handle.is_empty() {
        Err(Custom(Status::BadRequest, Json(ErrorInfo::new("Handle too short".into()))))
    } else {
        Ok(())
    }
}

fn validate_password(password: &str) -> Result<(), CustomError> {
    if password.len() >= 8 {
        Err(Custom(Status::BadRequest, Json(ErrorInfo::new("Password should be equal to or longer than 8".into()))))
    } else {
        Ok(())
    }
}

fn validate_nickname(nickname: &str) -> Result<(), CustomError> {
    if nickname.is_empty() {
        Err(Custom(Status::BadRequest, Json(ErrorInfo::new("Nickname too short".into()))))
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

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let mut conn = match manager.connect() {
            Ok(conn) => conn,
            Err(_) => {
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
) -> Result<Json<User>, CustomError> {
    use crate::schema::users;

    let mut conn = pool.get().expect("Failed to get database connection");

    validate_password(&update_user.password)?;

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

    Ok(Json(diesel::update(users::table)
        .set(update_user.into_inner().hash_password().map_err(|_| {
            Custom(
                Status::InternalServerError,
                Json(ErrorInfo::new("Failed to hash password".to_string())),
            )
        })?)
        .get_result::<UserSensitive>(&mut conn)
        .map_err(handle_error)?
        .into()))
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