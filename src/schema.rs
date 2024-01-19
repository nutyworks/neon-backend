// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "bundle_type"))]
    pub struct BundleType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "link_type"))]
    pub struct LinkType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "role_type"))]
    pub struct RoleType;
}

diesel::table! {
    artists (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        account_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::BundleType;

    bundles (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        price -> Nullable<Int4>,
        #[max_length = 255]
        description -> Nullable<Varchar>,
        #[sql_name = "type"]
        type_ -> BundleType,
        count -> Int4,
    }
}

diesel::table! {
    categories (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    characters (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        reference_id -> Int4,
    }
}

diesel::table! {
    circle_artists (id) {
        circle_id -> Int4,
        artist_id -> Int4,
        id -> Int4,
    }
}

diesel::table! {
    circle_bundles (id) {
        circle_id -> Int4,
        bundle_id -> Int4,
        id -> Int4,
    }
}

diesel::table! {
    circle_goods (id) {
        circle_id -> Int4,
        goods_id -> Int4,
        id -> Int4,
    }
}

diesel::table! {
    circle_links (id) {
        link_id -> Int4,
        circle_id -> Int4,
        id -> Int4,
    }
}

diesel::table! {
    circles (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        description -> Nullable<Text>,
        #[max_length = 255]
        location -> Nullable<Varchar>,
    }
}

diesel::table! {
    goods (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        description -> Nullable<Text>,
        price -> Nullable<Int4>,
        category_id -> Int4,
        #[max_length = 16]
        image_name -> Nullable<Bpchar>,
    }
}

diesel::table! {
    goods_character (id) {
        goods_id -> Int4,
        character_id -> Int4,
        id -> Int4,
    }
}

diesel::table! {
    goods_in_bundle (id) {
        bundle_id -> Int4,
        goods_id -> Int4,
        id -> Int4,
        count -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LinkType;

    links (id) {
        id -> Int4,
        #[sql_name = "type"]
        type_ -> LinkType,
        #[max_length = 255]
        url -> Varchar,
        name -> Nullable<Varchar>,
    }
}

diesel::table! {
    refs (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    tokens (id) {
        id -> Int4,
        #[max_length = 12]
        selector -> Bpchar,
        #[max_length = 64]
        hashed_validator -> Bpchar,
        user_id -> Int4,
        expires -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_circles (id) {
        id -> Int4,
        user_id -> Int4,
        circle_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RoleType;

    users (id) {
        id -> Int4,
        #[max_length = 20]
        handle -> Varchar,
        #[max_length = 100]
        nickname -> Varchar,
        #[max_length = 97]
        password -> Bpchar,
        #[max_length = 16]
        twitter_id -> Nullable<Varchar>,
        role -> RoleType,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 128]
        code_verifier -> Nullable<Bpchar>,
        #[max_length = 16]
        oauth_state -> Nullable<Bpchar>,
    }
}

diesel::joinable!(characters -> refs (reference_id));
diesel::joinable!(circle_artists -> artists (artist_id));
diesel::joinable!(circle_artists -> circles (circle_id));
diesel::joinable!(circle_bundles -> bundles (bundle_id));
diesel::joinable!(circle_bundles -> circles (circle_id));
diesel::joinable!(circle_goods -> circles (circle_id));
diesel::joinable!(circle_goods -> goods (goods_id));
diesel::joinable!(circle_links -> circles (circle_id));
diesel::joinable!(circle_links -> links (link_id));
diesel::joinable!(goods -> categories (category_id));
diesel::joinable!(goods_character -> characters (character_id));
diesel::joinable!(goods_character -> goods (goods_id));
diesel::joinable!(goods_in_bundle -> bundles (bundle_id));
diesel::joinable!(goods_in_bundle -> goods (goods_id));
diesel::joinable!(tokens -> users (user_id));
diesel::joinable!(user_circles -> circles (circle_id));
diesel::joinable!(user_circles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    artists,
    bundles,
    categories,
    characters,
    circle_artists,
    circle_bundles,
    circle_goods,
    circle_links,
    circles,
    goods,
    goods_character,
    goods_in_bundle,
    links,
    refs,
    tokens,
    user_circles,
    users,
);
