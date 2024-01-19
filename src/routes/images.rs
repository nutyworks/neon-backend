use rocket::fs::NamedFile;
use rocket::response::status::{Created, Custom};

use rocket::http::{ContentType, Status};
use rocket::Data;

use rocket::serde::json::Json;
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};

use std::fs::File;

use crate::error_handler::{ErrorInfo, CustomError};
use crate::models::AuthenticatedUser;
use crate::utils::strings::generate_random_string;

#[post("/images", data = "<data>")]
pub async fn upload_image(
    user: AuthenticatedUser,
    content_type: &ContentType, 
    data: Data<'_>
) -> Result<Created<()>, CustomError> {
    user.check_artist()?;

    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::raw("image")
            .size_limit(20000000)
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .map_err(|_| Custom(Status::InternalServerError, Json(ErrorInfo::new("internal server error".into()))))?
    ]);

    let multipart_form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .map_err(|_| Custom(Status::UnprocessableEntity, Json(ErrorInfo::new("cannot parse multipart data".into()))))?;

    let image_fields = multipart_form_data.raw.get("image");

    let raw_image = if let Some(file_fields) = image_fields {
        let file_field = &file_fields[0]; // Because we only put one "photo" field to the allowed_fields, the max length of this file_fields is 1.
        &file_field.raw
    } else {
        return Err(Custom(Status::BadRequest, Json(ErrorInfo::new("image field not found".into()))));
    };

    let img = image::load_from_memory(raw_image)
        .map_err(|_| Custom(Status::BadRequest, Json(ErrorInfo::new("provided file type is not supported".into()))))?;

    let resized_img = if img.width() > 720 {
        let ratio = 720.0 / img.width() as f32;
        img.resize(720, (img.height() as f32 * ratio) as u32, image::imageops::FilterType::Lanczos3)
    } else {
        img.to_owned()
    };

    let filename = generate_random_string(16);

    std::fs::create_dir_all(format!("images/{}", &filename[..2]))
        .map_err(|_| Custom(Status::InternalServerError, Json(ErrorInfo::new("internal server error".into()))))?;
    
    let mut file = File::create(format!("images/{}/{}.webp", &filename[..2], filename))
        .map_err(|_| Custom(Status::InternalServerError, Json(ErrorInfo::new("internal server error".into()))))?;

    resized_img.write_to(&mut file, image::ImageOutputFormat::WebP).unwrap();

    Ok(Created::new(format!("images/{}.webp", filename)))
}

#[get("/images/<filename>")]
pub async fn get_image(filename: String) -> Result<NamedFile, CustomError> {
    let is_valid_filename = filename.chars().all(|c| 
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".contains(c))
        && filename.len() == 16;

    if !is_valid_filename {
        return Err(Custom(Status::BadRequest, Json(ErrorInfo::new("invalid filename".into()))));
    }

    Ok(
        NamedFile::open(format!("images/{}/{}.webp", &filename[..2], filename))
            .await
            .map_err(|_| Custom(Status::NotFound, Json(ErrorInfo::new("file not found".into()))))?
    )
}