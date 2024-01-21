use std::collections::HashMap;
use std::time::SystemTime;

use crate::error_handler::{handle_error, CustomError, ErrorInfo};
use crate::models::{AuthenticatedUser, LinkTypeEnum, Link};
use crate::schema::sql_types::LinkType;
use crate::{models::Circle, DbPool};

use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text, Nullable, Timestamp};
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use serde::Serialize;

#[get("/circles?<name>&<artist_name>&<location>")]
pub fn get_circles(
    name: Option<String>,
    artist_name: Option<String>,
    location: Option<String>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<Circle>>, CustomError> {
    use crate::schema::artists;
    use crate::schema::circle_artists;
    use crate::schema::circles;

    let mut conn = pool.get().expect("Failed to get database connection");

    let mut query = circles::table
        .left_join(circle_artists::table.on(circles::id.eq(circle_artists::circle_id)))
        .left_join(artists::table.on(circle_artists::artist_id.eq(artists::id)))
        .into_boxed();

    if let Some(name) = name {
        query = query.filter(circles::name.similar_to(format!("%{}%", name)));
    }

    if let Some(artist_name) = artist_name {
        query = query.filter(artists::name.similar_to(format!("%{}%", artist_name)));
    }

    if let Some(location) = location {
        query = query.filter(circles::location.like(format!("%{}%", location)));
    }

    query
        .select(circles::all_columns)
        .distinct()
        .load::<Circle>(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[get("/circles/<circle_id>")]
pub fn get_circle_by_id(
    circle_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Circle>, CustomError> {
    use crate::schema::circles;

    let mut conn = pool.get().expect("Failed to get database connection");

    circles::dsl::circles
        .find(circle_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[derive(Queryable)]
pub struct CircleLinkRecord {
    pub circle_id: i32,
    pub circle_name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub link_id: i32,
    pub link_name: Option<String>,
    pub link_type: LinkTypeEnum,
    pub link_url: String,
    pub link_expire: Option<SystemTime>,
}

#[derive(Serialize)]
pub struct CircleLinks {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub links: Vec<Link>,
}

#[get("/circles/has_prepayment")]
pub fn get_circles_with_prepayment(
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<CircleLinks>>, CustomError> {
    use crate::schema::circles;
    use crate::schema::circle_links;
    use crate::schema::links;

    let mut conn = pool.get().expect("Failed to get database connection");

    let records = links::table
        .filter(links::type_.eq(LinkTypeEnum::prepayment))
        .inner_join(circle_links::table.on(links::id.eq(circle_links::link_id)))
        .inner_join(circles::table.on(circle_links::circle_id.eq(circles::id)))
        .select((
            sql::<Integer>("circles.id"),
            sql::<Nullable<Text>>("circles.name"),
            sql::<Nullable<Text>>("circles.description"),
            sql::<Nullable<Text>>("circles.location"),
            sql::<Integer>("links.id"),
            sql::<Nullable<Text>>("links.name"),
            sql::<LinkType>("links.type"),
            sql::<Text>("links.url"),
            sql::<Nullable<Timestamp>>("links.expire"),
        ))
        .load::<CircleLinkRecord>(&mut conn)
        .map_err(handle_error)?;

    let mut cl = HashMap::new();

    for record in records {
        let entry = cl.entry(record.circle_id).or_insert(CircleLinks {
            id: record.circle_id,
            name: record.circle_name,
            description: record.description,
            location: record.location,
            links: vec![],
        });

        entry.links.append(&mut vec![Link { 
            id: record.link_id,
            name: record.link_name,
            type_: record.link_type,
            url: record.link_url,
            expire: record.link_expire,
        }]);
    }

    let circles = cl.into_values().collect();

    Ok(Json(circles))
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::circles)]
pub struct NewCircle {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::circles)]
pub struct UpdateCircle {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
}

#[post("/circles", format = "json", data = "<new_circle>")]
pub fn post_circle(
    user: AuthenticatedUser,
    new_circle: Json<NewCircle>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Circle>>, CustomError> {
    use crate::schema::circles;

    user.check_moderator()?;

    let mut conn = pool.get().expect("Failed to get database connection");

    let circle = diesel::insert_into(circles::dsl::circles)
        .values(new_circle.into_inner())
        .get_result::<Circle>(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!("/circles/{}", circle.id)).body(Json(circle)))
}

#[patch("/circles/<circle_id>", format = "json", data = "<update_circle>")]
pub fn patch_circle(
    user: AuthenticatedUser,
    circle_id: i32,
    update_circle: Json<UpdateCircle>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Circle>, CustomError> {
    use crate::schema::circles::dsl::*;

    user.check_permission(circle_id)?;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::update(circles.find(circle_id))
        .set(update_circle.into_inner())
        .execute(&mut conn)
        .map_err(handle_error)?;

    circles
        .find(circle_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[delete("/circles/<circle_id>")]
pub fn delete_circle(
    user: AuthenticatedUser,
    circle_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::circle_artists;
    use crate::schema::circle_bundles;
    use crate::schema::circle_goods;
    use crate::schema::circle_links;
    use crate::schema::circles::dsl::*;

    user.check_moderator()?;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::delete(
        circle_artists::dsl::circle_artists.filter(circle_artists::dsl::circle_id.eq(circle_id)),
    )
    .execute(&mut conn)
    .map_err(handle_error)?;

    diesel::delete(
        circle_bundles::dsl::circle_bundles.filter(circle_bundles::dsl::circle_id.eq(circle_id)),
    )
    .execute(&mut conn)
    .map_err(handle_error)?;

    diesel::delete(
        circle_goods::dsl::circle_goods.filter(circle_goods::dsl::circle_id.eq(circle_id)),
    )
    .execute(&mut conn)
    .map_err(handle_error)?;

    diesel::delete(
        circle_links::dsl::circle_links.filter(circle_links::dsl::circle_id.eq(circle_id)),
    )
    .execute(&mut conn)
    .map_err(handle_error)?;

    let size = diesel::delete(circles.find(circle_id))
        .execute(&mut conn)
        .map_err(handle_error)?;

    if size == 0 {
        Err(Custom(
            Status::NotFound,
            Json(ErrorInfo::new("not_found".to_string())),
        ))
    } else {
        Ok(())
    }
}
