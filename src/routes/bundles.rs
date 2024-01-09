use crate::error_handler::{handle_error, CustomError, ErrorInfo};
use crate::models::{Bundle, BundleGoods, BundleTypeEnum};
use crate::DbPool;

use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;

use rocket::serde::Deserialize;

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::bundles)]
pub struct NewBundle {
    pub name: Option<String>,
    pub price: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: BundleTypeEnum,
    pub count: i32,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::bundles)]
pub struct UpdateBundle {
    pub name: Option<String>,
    pub price: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<BundleTypeEnum>,
    pub count: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::circle_bundles)]
pub struct NewCircleBundle {
    pub circle_id: i32,
    pub bundle_id: i32,
}

#[post("/circles/<circle_id>/bundles", format = "json", data = "<new_bundle>")]
pub fn post_circle_bundle(
    circle_id: i32,
    new_bundle: Json<NewBundle>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Bundle>>, CustomError> {
    use crate::schema::bundles;
    use crate::schema::circle_bundles;

    let mut conn = pool.get().expect("Failed to get database connection");

    let bundle = diesel::insert_into(bundles::dsl::bundles)
        .values(new_bundle.into_inner())
        .get_result::<Bundle>(&mut conn)
        .map_err(handle_error)?;

    diesel::insert_into(circle_bundles::dsl::circle_bundles)
        .values(NewCircleBundle {
            circle_id,
            bundle_id: bundle.id,
        })
        .execute(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!("/bundles/{}", bundle.id)).body(Json(bundle)))
}

#[get("/bundles?<circle_id>")]
pub fn get_bundles(
    circle_id: Option<i32>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<Bundle>>, CustomError> {
    use crate::schema::bundles;
    use crate::schema::circle_bundles;

    let mut conn = pool.get().expect("Failed to get database connection");

    let mut query = bundles::table
        .left_join(circle_bundles::table.on(bundles::id.eq(circle_bundles::circle_id)))
        .into_boxed();

    if let Some(circle_id) = circle_id {
        query = query.filter(circle_bundles::circle_id.eq(circle_id));
    }

    query
        .select(bundles::all_columns)
        .distinct()
        .load::<Bundle>(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[get("/bundles/<bundle_id>")]
pub fn get_bundle_by_id(
    bundle_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Bundle>, CustomError> {
    use crate::schema::bundles::dsl::bundles;

    let mut conn = pool.get().expect("Failed to get database connection");

    let bundle = bundles
        .find(bundle_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)?;

    Ok(bundle)
}

#[patch("/bundles/<bundle_id>", format = "json", data = "<update_bundle>")]
pub fn patch_bundle(
    bundle_id: i32,
    update_bundle: Json<UpdateBundle>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Bundle>, CustomError> {
    use crate::schema::bundles::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::update(bundles.find(bundle_id))
        .set(update_bundle.into_inner())
        .execute(&mut conn)
        .map_err(handle_error)?;

    bundles
        .find(bundle_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[delete("/bundles/<bundle_id>")]
pub fn delete_bundle(bundle_id: i32, pool: &rocket::State<DbPool>) -> Result<(), CustomError> {
    use crate::schema::bundles::dsl::*;
    use crate::schema::circle_bundles;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::delete(
        circle_bundles::dsl::circle_bundles.filter(circle_bundles::dsl::bundle_id.eq(bundle_id)),
    )
    .execute(&mut conn)
    .map_err(handle_error)?;

    let size = diesel::delete(bundles.find(bundle_id))
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
#[diesel(table_name = crate::schema::goods_in_bundle)]
pub struct NewGoodId {
    pub goods_id: i32,
    pub count: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::goods_in_bundle)]
pub struct NewBundleGoods {
    pub bundle_id: i32,
    pub goods_id: i32,
    pub count: i32,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::goods_in_bundle)]
pub struct UpdateBundleGoods {
    pub count: Option<i32>,
}

#[post("/bundles/<bundle_id>/goods", format = "json", data = "<new_goods>")]
pub fn post_bundle_goods(
    bundle_id: i32,
    new_goods: Json<NewGoodId>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<()>, CustomError> {
    use crate::schema::goods_in_bundle;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::insert_into(goods_in_bundle::dsl::goods_in_bundle)
        .values(NewBundleGoods {
            bundle_id,
            goods_id: new_goods.goods_id,
            count: new_goods.count,
        })
        .execute(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!(
        "/bundles/{}/goods/{}",
        bundle_id, new_goods.goods_id
    )))
}

#[patch(
    "/bundles/<bundle_id>/goods/<goods_id>",
    format = "json",
    data = "<update_bundle_goods>"
)]
pub fn patch_bundle_goods(
    bundle_id: i32,
    goods_id: i32,
    update_bundle_goods: Json<UpdateBundleGoods>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<BundleGoods>, CustomError> {
    use crate::schema::goods_in_bundle;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::update(
        goods_in_bundle::dsl::goods_in_bundle
            .filter(goods_in_bundle::dsl::bundle_id.eq(bundle_id))
            .filter(goods_in_bundle::dsl::goods_id.eq(goods_id)),
    )
    .set(update_bundle_goods.into_inner())
    .execute(&mut conn)
    .map_err(handle_error)?;

    let updated_goods_in_bundle = goods_in_bundle::dsl::goods_in_bundle
        .filter(goods_in_bundle::dsl::bundle_id.eq(bundle_id))
        .filter(goods_in_bundle::dsl::goods_id.eq(goods_id))
        .first(&mut conn)
        .map_err(handle_error)?;

    Ok(Json(updated_goods_in_bundle))
}

#[delete("/bundles/<bundle_id>/goods/<goods_id>")]
pub fn delete_bundle_goods(
    bundle_id: i32,
    goods_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::goods_in_bundle;

    let mut conn = pool.get().expect("Failed to get database connection");

    let size = diesel::delete(
        goods_in_bundle::dsl::goods_in_bundle
            .filter(goods_in_bundle::dsl::bundle_id.eq(bundle_id))
            .filter(goods_in_bundle::dsl::goods_id.eq(goods_id)),
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
