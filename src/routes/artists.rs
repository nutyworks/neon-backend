use crate::error_handler::{handle_error, CustomError, ErrorInfo};
use crate::models::Artist;
use crate::DbPool;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;

use rocket::serde::Deserialize;

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::artists)]
pub struct NewArtist {
    pub name: String,
    pub account_url: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::artists)]
pub struct UpdateArtist {
    pub name: Option<String>,
    pub account_url: Option<String>,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::circle_artists)]
pub struct NewArtistId {
    pub artist_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::circle_artists)]
pub struct NewCircleArtist {
    pub circle_id: i32,
    pub artist_id: i32,
}

#[post(
    "/circles/<circle_id>/artists",
    format = "json",
    data = "<new_artist_id>"
)]
pub fn post_circle_artist(
    circle_id: i32,
    new_artist_id: Json<NewArtistId>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<()>, CustomError> {
    use crate::schema::circle_artists;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::insert_into(circle_artists::dsl::circle_artists)
        .values(NewCircleArtist {
            circle_id,
            artist_id: new_artist_id.artist_id,
        })
        .execute(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!(
        "/circles/{}/artists/{}",
        circle_id, new_artist_id.artist_id
    )))
}

#[delete("/circles/<circle_id>/artists/<artist_id>")]
pub fn delete_circle_artist(
    circle_id: i32,
    artist_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::circle_artists;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::delete(
        circle_artists::dsl::circle_artists
            .filter(circle_artists::dsl::circle_id.eq(circle_id))
            .filter(circle_artists::dsl::artist_id.eq(artist_id)))
        .execute(&mut conn)
        .map_err(handle_error)?;

    Ok(())
}

#[post("/artists", format = "json", data = "<new_artist>")]
pub fn post_artist(
    new_artist: Json<NewArtist>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Artist>>, CustomError> {
    use crate::schema::artists;

    let mut conn = pool.get().expect("Failed to get database connection");

    let artist = diesel::insert_into(artists::dsl::artists)
        .values(new_artist.into_inner())
        .get_result::<Artist>(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!("/artists/{}", artist.id)).body(Json(artist)))
}

#[get("/artists")]
pub fn get_artists(pool: &rocket::State<DbPool>) -> Result<Json<Vec<Artist>>, CustomError> {
    use crate::schema::artists::dsl::artists;

    let mut conn = pool.get().expect("Failed to get database connection");

    artists
        .load::<Artist>(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[get("/artists/<artist_id>")]
pub fn get_artist_by_id(
    artist_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Artist>, Custom<String>> {
    use crate::schema::artists::dsl::artists;

    let mut conn = pool.get().expect("Failed to get database connection");

    match artists.find(artist_id).first(&mut conn) {
        Ok(artist) => Ok(Json(artist)),
        Err(_) => Err(Custom(Status::NotFound, "Artist not found".to_string())),
    }
}

#[patch("/artists/<artist_id>", format = "json", data = "<update_artist>")]
pub fn patch_artist(
    artist_id: i32,
    update_artist: Json<UpdateArtist>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Artist>, CustomError> {
    use crate::schema::artists::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::update(artists.find(artist_id))
        .set(update_artist.into_inner())
        .execute(&mut conn)
        .map_err(handle_error)?;

    artists
        .find(artist_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[delete("/artists/<artist_id>")]
pub fn delete_artist(
    artist_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::artists::dsl::*;

    let mut conn = pool.get().expect("Failed to get database connection");

    let size = diesel::delete(artists.find(artist_id))
        .execute(&mut conn)
        .map_err(handle_error)?;

    if size == 0 {
        Err(Custom(Status::NotFound, Json(ErrorInfo::new("not_found".to_string()))))
    } else {
        Ok(())
    }
}
