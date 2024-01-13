#[macro_use]
extern crate rocket;
extern crate diesel;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::Response;
use rocket::{http::Status, Request};
use serde_json::{json, Value};
use std::env;

use routes::artists::{
    delete_artist, delete_circle_artist, get_artist_by_id, get_artists, patch_artist, post_artist,
    post_circle_artist,
};
use routes::bundles::{
    delete_bundle, delete_bundle_goods, get_bundle_by_id, get_bundles, patch_bundle,
    patch_bundle_goods, post_bundle_goods, post_circle_bundle,
};
use routes::categories::{
    delete_category, get_categories, get_category_by_id, patch_category, post_category,
};
use routes::characters::{
    delete_character, get_character_by_id, get_characters, patch_character, post_character,
};
use routes::circles::{delete_circle, get_circle_by_id, get_circles, patch_circle, post_circle};
use routes::goods::{
    delete_good_character, delete_goods, get_goods, get_goods_by_id, patch_goods,
    post_circle_goods, post_good_character,
};
use routes::links::{delete_link, get_link_by_id, get_links, patch_link, post_circle_link};
use routes::references::{
    delete_reference, get_reference_by_id, get_references, patch_reference, post_reference,
};

mod error_handler;
mod models;
mod routes;
mod schema;

type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct CORS;

#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, DELETE, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create database pool");

    let rocket = rocket::build()
        .manage(pool)
        .mount(
            "/",
            routes![
                get_artists,
                get_artist_by_id,
                post_artist,
                post_circle_artist,
                patch_artist,
                delete_artist,
                delete_circle_artist,
                get_circles,
                get_circle_by_id,
                post_circle,
                patch_circle,
                delete_circle,
                post_circle_goods,
                get_goods,
                get_goods_by_id,
                patch_goods,
                delete_goods,
                post_good_character,
                delete_good_character,
                post_circle_bundle,
                get_bundles,
                get_bundle_by_id,
                patch_bundle,
                delete_bundle,
                post_bundle_goods,
                delete_bundle_goods,
                post_character,
                get_characters,
                get_character_by_id,
                patch_character,
                delete_character,
                post_reference,
                get_references,
                get_reference_by_id,
                patch_reference,
                delete_reference,
                post_category,
                get_categories,
                get_category_by_id,
                patch_category,
                delete_category,
                post_circle_link,
                get_links,
                get_link_by_id,
                patch_link,
                delete_link,
                patch_bundle_goods,
                all_options,
            ],
        )
        .register("/", catchers![catch_default]);

    if cfg!(debug_assertions) {
        rocket.attach(CORS)
    } else {
        rocket
    }
}

#[catch(default)]
pub fn catch_default(status: Status, _req: &Request) -> Value {
    json!(
        {
            "success": false,
            "code": status.code
        }
    )
}
