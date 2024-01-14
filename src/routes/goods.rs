use crate::error_handler::{handle_error, CustomError, ErrorInfo};
use crate::models::{AuthenticatedUser, Good};
use crate::DbPool;

use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::goods)]
pub struct NewGood {
    pub name: String,
    pub description: Option<String>,
    pub price: Option<i32>,
    pub category_id: i32,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::goods)]
pub struct UpdateGood {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<i32>,
    pub category_id: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::circle_goods)]
pub struct NewCircleGoods {
    pub circle_id: i32,
    pub goods_id: i32,
}

#[post("/circles/<circle_id>/goods", format = "json", data = "<new_goods>")]
pub fn post_circle_goods(
    user: AuthenticatedUser,
    circle_id: i32,
    new_goods: Json<NewGood>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Good>>, CustomError> {
    use crate::schema::circle_goods;
    use crate::schema::goods;

    user.check_permission(circle_id)?;

    let mut conn = pool.get().expect("Failed to get database connection");

    let good = diesel::insert_into(goods::dsl::goods)
        .values(new_goods.into_inner())
        .get_result::<Good>(&mut conn)
        .map_err(handle_error)?;

    let size = diesel::insert_into(circle_goods::dsl::circle_goods)
        .values(NewCircleGoods {
            circle_id,
            goods_id: good.id,
        })
        .execute(&mut conn)
        .map_err(handle_error)?;

    if size == 0 {
        Err(Custom(
            Status::NotFound,
            Json(ErrorInfo::new("not_found".to_string())),
        ))
    } else {
        Ok(Created::new(format!("/goods/{}", good.id)).body(Json(good)))
    }
}

#[get("/goods?<name>&<character_id>&<ref_id>&<bundle_id>&<circle_id>")]
pub fn get_goods(
    name: Option<String>,
    character_id: Option<i32>,
    ref_id: Option<i32>,
    bundle_id: Option<i32>,
    circle_id: Option<i32>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<Good>>, CustomError> {
    use crate::schema::characters;
    use crate::schema::circle_goods;
    use crate::schema::goods;
    use crate::schema::goods_character;
    use crate::schema::goods_in_bundle;

    let mut conn = pool.get().expect("Failed to get database connection");

    // Start with the base query
    let mut query = goods::dsl::goods
        .left_join(goods_character::table.on(goods::id.eq(goods_character::goods_id)))
        .left_join(characters::table.on(goods_character::character_id.eq(characters::id)))
        .left_join(goods_in_bundle::table.on(goods::id.eq(goods_in_bundle::goods_id)))
        .left_join(circle_goods::table.on(goods::id.eq(circle_goods::goods_id)))
        .into_boxed();

    // Apply filters based on query parameters
    if let Some(name_filter) = name {
        query = query.filter(goods::name.like(format!("%{}%", name_filter)));
    }

    if let Some(character_id_filter) = character_id {
        // Apply character_id filter
        query = query.filter(goods_character::dsl::character_id.eq(character_id_filter));
    }

    if let Some(ref_id_filter) = ref_id {
        // Apply ref_id filter
        query = query.filter(characters::dsl::reference_id.eq(ref_id_filter));
    }

    if let Some(bundle_id_filter) = bundle_id {
        // Apply bundle_id filter
        query = query.filter(goods_in_bundle::dsl::bundle_id.eq(bundle_id_filter));
    }

    if let Some(circle_id_filter) = circle_id {
        // Apply circle_id filter
        query = query.filter(circle_goods::dsl::circle_id.eq(circle_id_filter));
    }

    // Execute the final query and return the result
    query
        .select(goods::all_columns)
        .distinct()
        .load::<Good>(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[get("/goods/<goods_id>")]
pub fn get_goods_by_id(
    goods_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Good>, CustomError> {
    use crate::schema::goods::dsl::goods;

    let mut conn = pool.get().expect("Failed to get database connection");

    goods
        .find(goods_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[patch("/goods/<goods_id>", format = "json", data = "<update_goods>")]
pub fn patch_goods(
    user: AuthenticatedUser,
    goods_id: i32,
    update_goods: Json<UpdateGood>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Good>, CustomError> {
    use crate::schema::circle_goods;
    use crate::schema::goods::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    let circle_id = circle_goods::table
        .filter(circle_goods::goods_id.eq(goods_id))
        .select(circle_goods::circle_id)
        .first::<i32>(&mut conn)
        .map_err(handle_error)?;

    user.check_permission(circle_id)?;

    diesel::update(goods.find(goods_id))
        .set(update_goods.into_inner())
        .execute(&mut conn)
        .map_err(handle_error)?;

    goods
        .find(goods_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[delete("/goods/<goods_id>")]
pub fn delete_goods(
    user: AuthenticatedUser,
    goods_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::circle_goods;
    use crate::schema::goods::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    let circle_id = circle_goods::table
        .filter(circle_goods::goods_id.eq(goods_id))
        .select(circle_goods::circle_id)
        .first::<i32>(&mut conn)
        .map_err(handle_error)?;

    user.check_permission(circle_id)?;

    diesel::delete(
        circle_goods::dsl::circle_goods.filter(circle_goods::dsl::goods_id.eq(goods_id)),
    )
    .execute(&mut conn)
    .map_err(handle_error)?;

    let size = diesel::delete(goods.find(goods_id))
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

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::goods_character)]
pub struct NewCharacterId {
    pub character_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::goods_character)]
pub struct NewGoodsCharacter {
    pub goods_id: i32,
    pub character_id: i32,
}

#[post(
    "/goods/<goods_id>/characters",
    format = "json",
    data = "<new_character_id>"
)]
pub fn post_good_character(
    user: AuthenticatedUser,
    goods_id: i32,
    new_character_id: Json<NewCharacterId>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<()>, CustomError> {
    use crate::schema::circle_goods;
    use crate::schema::goods_character;

    let mut conn = pool.get().expect("Failed to get database connection");

    let circle_id = circle_goods::table
        .filter(circle_goods::goods_id.eq(goods_id))
        .select(circle_goods::circle_id)
        .first::<i32>(&mut conn)
        .map_err(handle_error)?;

    user.check_permission(circle_id)?;

    diesel::insert_into(goods_character::dsl::goods_character)
        .values(NewGoodsCharacter {
            goods_id,
            character_id: new_character_id.character_id,
        })
        .execute(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!(
        "/goods/{}/characters/{}",
        goods_id, new_character_id.character_id
    )))
}

#[delete("/goods/<goods_id>/characters/<character_id>")]
pub fn delete_good_character(
    user: AuthenticatedUser,
    goods_id: i32,
    character_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::circle_goods;
    use crate::schema::goods_character;

    let mut conn = pool.get().expect("Failed to get database connection");

    let circle_id = circle_goods::table
        .filter(circle_goods::goods_id.eq(goods_id))
        .select(circle_goods::circle_id)
        .first::<i32>(&mut conn)
        .map_err(handle_error)?;

    user.check_permission(circle_id)?;

    let size = diesel::delete(
        goods_character::dsl::goods_character
            .filter(goods_character::dsl::goods_id.eq(goods_id))
            .filter(goods_character::dsl::character_id.eq(character_id)),
    )
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
