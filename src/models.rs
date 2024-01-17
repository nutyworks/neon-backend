use std::time::SystemTime;

use diesel::prelude::Queryable;
use rocket::{
    http::Status,
    response::status::Custom,
    serde::{json::Json, Serialize},
};
use serde::Deserialize;

use crate::error_handler::{CustomError, ErrorInfo};

#[allow(non_camel_case_types)]
#[derive(diesel_derive_enum::DbEnum, Debug, Deserialize, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::BundleType"]
pub enum BundleTypeEnum {
    random,
    select,
}

#[allow(non_camel_case_types)]
#[derive(diesel_derive_enum::DbEnum, Debug, Deserialize, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::LinkType"]
pub enum LinkTypeEnum {
    info,
    other,
    netorder,
    notice,
    prepayment,
    demand,
}

#[allow(non_camel_case_types)]
#[derive(diesel_derive_enum::DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::RoleType"]
pub enum RoleTypeEnum {
    admin,
    moderator,
    user,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Circle {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Good {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<i32>,
    pub category_id: i32,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FullGood {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<i32>,
    pub category: Category,
    pub characters: Vec<CharacterWithReference>,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CharacterWithReference {
    pub id: i32,
    pub character: String,
    pub reference: String,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Bundle {
    pub id: i32,
    pub name: Option<String>,
    pub price: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: BundleTypeEnum,
    pub count: i32,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct BundleGoods {
    pub id: i32,
    pub bundle_id: i32,
    pub goods_id: i32,
    pub count: i32,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Character {
    pub id: i32,
    pub name: String,
    pub reference_id: i32,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Ref {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Category {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Artist {
    pub id: i32,
    pub name: String,
    pub account_url: Option<String>,
}

#[derive(Queryable, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Link {
    pub id: i32,
    #[serde(rename = "type")]
    pub type_: LinkTypeEnum,
    pub url: String,
    pub name: Option<String>,
}

#[derive(Queryable)]
pub struct UserSensitive {
    pub id: i32,
    pub handle: String,
    pub nickname: String,
    pub password: String,
    pub twitter_id: Option<String>,
    pub role: RoleTypeEnum,
    pub email: String,
    pub code_verifier: Option<String>,
    pub oauth_state: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub handle: String,
    pub nickname: String,
    pub twitter_id: Option<String>,
    pub role: RoleTypeEnum,
}

#[derive(Queryable, Serialize)]
pub struct AuthenticatedUser {
    pub id: i32,
    pub handle: String,
    pub nickname: String,
    pub twitter_id: Option<String>,
    pub email: String,
    pub role: RoleTypeEnum,
    pub circles: Vec<i32>,
}

impl AuthenticatedUser {
    pub fn check_permission(&self, circle_id: i32) -> Result<(), CustomError> {
        match self.role {
            RoleTypeEnum::admin | RoleTypeEnum::moderator => Ok(()),
            RoleTypeEnum::user => {
                if self.circles.contains(&circle_id) {
                    Ok(())
                } else {
                    Err(Custom(
                        Status::Unauthorized,
                        Json(ErrorInfo::new("You are not allowed to do this!".into())),
                    ))
                }
            }
        }
    }

    pub fn check_moderator(&self) -> Result<(), CustomError> {
        match self.role {
            RoleTypeEnum::admin | RoleTypeEnum::moderator => Ok(()),
            RoleTypeEnum::user => Err(Custom(
                Status::Unauthorized,
                Json(ErrorInfo::new("You are not allowed to do this!".into())),
            )),
        }
    }

    pub fn check_artist(&self) -> Result<(), CustomError> {
        match self.role {
            RoleTypeEnum::admin | RoleTypeEnum::moderator => Ok(()),
            RoleTypeEnum::user => {
                if self.circles.is_empty() {
                    Err(Custom(
                        Status::Unauthorized,
                        Json(ErrorInfo::new("You are not allowed to do this!".into())),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl Into<User> for UserSensitive {
    fn into(self) -> User {
        User {
            handle: self.handle,
            nickname: self.nickname,
            twitter_id: self.twitter_id,
            role: self.role,
        }
    }
}

#[derive(Queryable)]
pub struct Token {
    pub id: i32,
    pub selector: String,
    pub hashed_validator: String,
    pub user_id: i32,
    pub expires: Option<SystemTime>,
}
