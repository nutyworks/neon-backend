use crate::error_handler::{handle_error, CustomError, ErrorInfo};
use crate::models::Ref;
use crate::DbPool;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;

use rocket::serde::Deserialize;

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::refs)]
pub struct NewRef {
    pub name: String,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::refs)]
pub struct UpdateRef {
    pub name: Option<String>,
}

#[post("/references", format = "json", data = "<new_reference>")]
pub fn post_reference(
    new_reference: Json<NewRef>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Ref>>, CustomError> {
    use crate::schema::refs;

    let mut conn = pool.get().expect("Failed to get database connection");

    let reference = diesel::insert_into(refs::dsl::refs)
        .values(new_reference.into_inner())
        .get_result::<Ref>(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!("/references/{}", reference.id)).body(Json(reference)))
}

#[get("/references?<name>")]
pub fn get_references(
    name: Option<String>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<Ref>>, CustomError> {
    use crate::schema::refs;

    let mut conn = pool.get().expect("Failed to get database connection");

    refs::table
        .filter(refs::name.like(format!("%{}%", name.unwrap_or("".into()))))
        .load::<Ref>(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[get("/references/<ref_id>")]
pub fn get_reference_by_id(
    ref_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Ref>, CustomError> {
    use crate::schema::refs::dsl::refs;

    let mut conn = pool.get().expect("Failed to get database connection");

    refs.find(ref_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[patch("/references/<ref_id>", format = "json", data = "<update_reference>")]
pub fn patch_reference(
    ref_id: i32,
    update_reference: Json<UpdateRef>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Ref>, CustomError> {
    use crate::schema::refs::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::update(refs.find(ref_id))
        .set(update_reference.into_inner())
        .execute(&mut conn)
        .map_err(handle_error)?;

    refs
        .find(ref_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[delete("/references/<ref_id>")]
pub fn delete_reference(
    ref_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::refs::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    let size = diesel::delete(refs.find(ref_id))
        .execute(&mut conn)
        .map_err(handle_error)?;

    if size == 0 {
        Err(Custom(Status::NotFound, Json(ErrorInfo::new("not_found".to_string()))))
    } else {
        Ok(())
    }
}
