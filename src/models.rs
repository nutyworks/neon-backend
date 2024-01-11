use diesel::prelude::Queryable;
use rocket::serde::Serialize;
use serde::Deserialize;

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
