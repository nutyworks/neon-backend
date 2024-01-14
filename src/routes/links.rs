use crate::error_handler::{handle_error, CustomError, ErrorInfo};
use crate::models::{AuthenticatedUser, Link, LinkTypeEnum};
use crate::DbPool;

use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;
use serde::Deserialize;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::circle_links)]
pub struct NewCircleLink {
    pub circle_id: i32,
    pub link_id: i32,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::links)]
pub struct NewLink {
    #[serde(rename = "type")]
    pub type_: LinkTypeEnum,
    pub url: String,
    pub name: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::links)]
pub struct UpdateLink {
    #[serde(rename = "type")]
    pub type_: Option<LinkTypeEnum>,
    pub url: Option<String>,
    pub name: Option<String>,
}

#[post("/circles/<circle_id>/links", format = "json", data = "<new_link>")]
pub fn post_circle_link(
    user: AuthenticatedUser,
    circle_id: i32,
    new_link: Json<NewLink>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Link>>, CustomError> {
    use crate::schema::circle_links;
    use crate::schema::links;

    user.check_permission(circle_id)?;

    let mut conn = pool.get().expect("Failed to get database connection");

    let link = diesel::insert_into(links::dsl::links)
        .values(new_link.into_inner())
        .get_result::<Link>(&mut conn)
        .map_err(handle_error)?;

    diesel::insert_into(circle_links::dsl::circle_links)
        .values(NewCircleLink {
            circle_id,
            link_id: link.id,
        })
        .execute(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!("/links/{}", link.id)).body(Json(link)))
}

#[get("/links?<circle_id>")]
pub fn get_links(
    circle_id: Option<i32>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<Link>>, CustomError> {
    use crate::schema::circle_links;
    use crate::schema::links;

    let mut conn = pool.get().expect("Failed to get database connection");

    let mut query = links::table
        .left_join(circle_links::table.on(links::id.eq(circle_links::link_id)))
        .into_boxed();

    if let Some(circle_id) = circle_id {
        query = query.filter(circle_links::circle_id.eq(circle_id));
    }

    query
        .select(links::all_columns)
        .distinct()
        .load::<Link>(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[get("/links/<link_id>")]
pub fn get_link_by_id(
    link_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Link>, CustomError> {
    use crate::schema::links::dsl::links;

    let mut conn = pool.get().expect("Failed to get database connection");

    links
        .find(link_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[patch("/links/<link_id>", format = "json", data = "<link_request>")]
pub fn patch_link(
    user: AuthenticatedUser,
    link_id: i32,
    link_request: Json<UpdateLink>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Link>, CustomError> {
    use crate::schema::circle_links;
    use crate::schema::links::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    let circle_id = circle_links::table
        .filter(circle_links::link_id.eq(link_id))
        .select(circle_links::circle_id)
        .first::<i32>(&mut conn)
        .map_err(handle_error)?;

    user.check_permission(circle_id)?;

    diesel::update(links.find(link_id))
        .set(link_request.into_inner())
        .execute(&mut conn)
        .map_err(handle_error)?;

    links
        .find(link_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[delete("/links/<link_id>")]
pub fn delete_link(
    user: AuthenticatedUser,
    link_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::circle_links;
    use crate::schema::links::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    let circle_id = circle_links::table
        .filter(circle_links::link_id.eq(link_id))
        .select(circle_links::circle_id)
        .first::<i32>(&mut conn)
        .map_err(handle_error)?;

    user.check_permission(circle_id)?;

    diesel::delete(circle_links::dsl::circle_links.filter(circle_links::dsl::link_id.eq(link_id)))
        .execute(&mut conn)
        .map_err(handle_error)?;

    let size = diesel::delete(links.find(link_id))
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
