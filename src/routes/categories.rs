use crate::error_handler::{handle_error, CustomError, ErrorInfo};
use crate::models::Category;
use crate::DbPool;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;

use rocket::serde::Deserialize;

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::categories)]
pub struct NewCategory {
    pub name: String,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::categories)]
pub struct UpdateCategory {
    pub name: String,
}

#[post("/categories", format = "json", data = "<new_category>")]
pub fn post_category(
    new_category: Json<NewCategory>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Category>>, CustomError> {
    use crate::schema::categories;

    let mut conn = pool.get().expect("Failed to get database connection");

    let category = diesel::insert_into(categories::dsl::categories)
        .values(new_category.into_inner())
        .get_result::<Category>(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!("/categories/{}", category.id)).body(Json(category)))
}

#[get("/categories?<name>")]
pub fn get_categories(
    name: Option<String>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<Category>>, CustomError> {
    use crate::schema::categories;

    let mut conn = pool.get().expect("Failed to get database connection");

    categories::table
        .filter(categories::name.like(format!("%{}%", name.unwrap_or("".into()))))
        .load::<Category>(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[get("/categories/<category_id>")]
pub fn get_category_by_id(
    category_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Category>, CustomError> {
    use crate::schema::categories::dsl::categories;

    let mut conn = pool.get().expect("Failed to get database connection");

    let category = categories
        .find(category_id)
        .first(&mut conn)
        .map_err(handle_error)?;

    Ok(Json(category))
}

#[patch(
    "/categories/<category_id>",
    format = "json",
    data = "<update_category>"
)]
pub fn patch_category(
    category_id: i32,
    update_category: Json<UpdateCategory>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Category>, CustomError> {
    use crate::schema::categories::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::update(categories.find(category_id))
        .set(update_category.into_inner())
        .execute(&mut conn)
        .map_err(handle_error)?;

    categories
        .find(category_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[delete("/categories/<category_id>")]
pub fn delete_category(category_id: i32, pool: &rocket::State<DbPool>) -> Result<(), CustomError> {
    use crate::schema::categories::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    let size = diesel::delete(categories.find(category_id))
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
