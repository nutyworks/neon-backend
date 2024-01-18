use crate::error_handler::{handle_error, CustomError, ErrorInfo};
use crate::models::{AuthenticatedUser, Character, CharacterWithReference};
use crate::DbPool;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use rocket::http::Status;
use rocket::response::status::{Created, Custom};
use rocket::serde::json::Json;

use rocket::serde::Deserialize;

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::characters)]
pub struct NewCharacter {
    pub name: String,
    pub reference_id: i32,
}

#[derive(Queryable, Selectable, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::characters)]
pub struct UpdateCharacter {
    pub name: Option<String>,
    pub reference_id: Option<i32>,
}

#[post("/characters", format = "json", data = "<new_character>")]
pub fn post_character(
    user: AuthenticatedUser,
    new_character: Json<NewCharacter>,
    pool: &rocket::State<DbPool>,
) -> Result<Created<Json<Character>>, CustomError> {
    use crate::schema::characters;

    user.check_artist()?;

    let mut conn = pool.get().expect("Failed to get database connection");

    let character = diesel::insert_into(characters::dsl::characters)
        .values(new_character.into_inner())
        .get_result::<Character>(&mut conn)
        .map_err(handle_error)?;

    Ok(Created::new(format!("/characters/{}", character.id)).body(Json(character)))
}

#[get("/characters?<name>&<ref_id>&<ref_name>")]
pub fn get_characters(
    name: Option<String>,
    ref_id: Option<i32>,
    ref_name: Option<String>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Vec<CharacterWithReference>>, CustomError> {
    use crate::schema::characters;
    use crate::schema::refs;

    let mut conn = pool.get().expect("Failed to get database connection");

    let mut query = characters::table
        .left_join(refs::table.on(characters::reference_id.eq(refs::id)))
        .into_boxed();

    if let Some(name) = name {
        query = query.filter(characters::name.similar_to(format!("%{}%", name)));
    }

    if let Some(ref_id) = ref_id {
        query = query.filter(characters::reference_id.eq(ref_id));
    }

    if let Some(ref_name) = ref_name {
        query = query.filter(refs::name.eq(ref_name));
    }

    query
        .select((sql::<Integer>("characters.id"), sql::<Text>("characters.name"), sql::<Text>("refs.name")))
        .distinct()
        .load::<CharacterWithReference>(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[get("/characters/<character_id>")]
pub fn get_character_by_id(
    character_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Character>, CustomError> {
    use crate::schema::characters::dsl::characters;

    let mut conn = pool.get().expect("Failed to get database connection");

    characters
        .find(character_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[patch(
    "/characters/<character_id>",
    format = "json",
    data = "<update_character>"
)]
pub fn patch_character(
    user: AuthenticatedUser,
    character_id: i32,
    update_character: Json<UpdateCharacter>,
    pool: &rocket::State<DbPool>,
) -> Result<Json<Character>, CustomError> {
    use crate::schema::characters::dsl::*;

    user.check_moderator()?;

    let mut conn = pool.get().expect("Failed to get database connection");

    diesel::update(characters.find(character_id))
        .set(update_character.into_inner())
        .execute(&mut conn)
        .map_err(handle_error)?;

    characters
        .find(character_id)
        .first(&mut conn)
        .map(Json)
        .map_err(handle_error)
}

#[delete("/characters/<character_id>")]
pub fn delete_character(
    user: AuthenticatedUser,
    character_id: i32,
    pool: &rocket::State<DbPool>,
) -> Result<(), CustomError> {
    use crate::schema::characters::dsl::*;

    user.check_moderator()?;

    let mut conn = pool.get().expect("Failed to get database connection");

    let size = diesel::delete(characters.find(character_id))
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
