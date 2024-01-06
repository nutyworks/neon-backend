use crate::error_handler::{CustomError, handle_error, ErrorInfo};
use crate::{models::Circle, DbPool};

use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;

#[get("/circles?<name>&<artist_name>&<location>")]
pub fn get_circles(
    name: Option<String>,
    artist_name: Option<String>,
    location: Option<String>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<Circle>>, CustomError> {
    use crate::schema::circles;
    use crate::schema::circle_artists;
    use crate::schema::artists;

    let mut conn = pool.get().expect("Failed to get database connection");

    let mut query = circles::table
        .left_join(circle_artists::table.on(circles::id.eq(circle_artists::circle_id)))
        .left_join(artists::table.on(circle_artists::artist_id.eq(artists::id)))
        .into_boxed();

    if let Some(name) = name {
        query = query.filter(circles::name.like(format!("%{}%", name)));
    }

    if let Some(artist_name) = artist_name {
        query = query.filter(artists::name.like(format!("%{}%", artist_name)));
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

    circles::dsl::circles.find(circle_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::circles)]
pub struct NewCircle {
    pub name: String,
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
    new_circle: Json<NewCircle>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Circle>>, CustomError> {
    use crate::schema::circles;

    let mut conn = pool.get().expect("Failed to get database connection");

    let circle= diesel::insert_into(circles::dsl::circles)
        .values(new_circle.into_inner())
        .get_result::<Circle>(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!("/circles/{}", circle.id)).body(Json(circle)))
}

#[patch("/circles/<circle_id>", format = "json", data = "<update_circle>")]
pub fn patch_circle(
    circle_id: i32,
    update_circle: Json<UpdateCircle>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Circle>, CustomError> {
    use crate::schema::circles::dsl::*;

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
    circle_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::circles::dsl::*;
    use crate::schema::circle_artists;
    use crate::schema::circle_bundles;
    use crate::schema::circle_goods;
    use crate::schema::circle_links;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::delete(
        circle_artists::dsl::circle_artists.filter(
            circle_artists::dsl::circle_id.eq(circle_id)
        ))
        .execute(&mut conn)
        .map_err(handle_error)?;

    diesel::delete(
        circle_bundles::dsl::circle_bundles.filter(
            circle_bundles::dsl::circle_id.eq(circle_id)
        ))
        .execute(&mut conn)
        .map_err(handle_error)?;

    diesel::delete(
        circle_goods::dsl::circle_goods.filter(
            circle_goods::dsl::circle_id.eq(circle_id)
        ))
        .execute(&mut conn)
        .map_err(handle_error)?;

    diesel::delete(
        circle_links::dsl::circle_links.filter(
            circle_links::dsl::circle_id.eq(circle_id)
        ))
        .execute(&mut conn)
        .map_err(handle_error)?;

    let size = diesel::delete(circles.find(circle_id))
        .execute(&mut conn)
        .map_err(handle_error)?;

    if size == 0 {
        Err(Custom(Status::NotFound, Json(ErrorInfo::new("not_found".to_string()))))
    } else {
        Ok(())
    }
}
