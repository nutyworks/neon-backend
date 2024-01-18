use rocket::response::status::Created;

use rocket::http::ContentType;
use rocket::Data;

use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};

use std::fs::File;

use std::time::{Instant, Duration};


#[post("/images", data = "<data>")]
pub async fn upload_image(content_type: &ContentType, data: Data<'_>) -> Result<Created<()>, String> {
    // let bytes = data.open(20.megabytes()).into_bytes().await.unwrap();
    // println!("{:?}", String::from_utf8_lossy(&bytes));

    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::raw("image")
            .size_limit(20000000)
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .unwrap(),
    ]);

    let start_time = Instant::now();

    let multipart_form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .unwrap();

    let end_time = Instant::now();
    println!("Phase 1: {} milliseconds", (end_time - start_time).as_millis());

    let image_fields = multipart_form_data.raw.get("image");

    let raw_image = if let Some(file_fields) = image_fields {
        let file_field = &file_fields[0]; // Because we only put one "photo" field to the allowed_fields, the max length of this file_fields is 1.
        &file_field.raw
    } else {
        unimplemented!("add error info");
    };

    println!("Phase 1.5: {} milliseconds", (end_time - start_time).as_millis());

    let img = image::load_from_memory(raw_image).expect("not image");

    let end_time = Instant::now();
    println!("Phase 2: {} milliseconds", (end_time - start_time).as_millis());

    let resized_img = if img.width() > 720 {
        let ratio = 720.0 / img.width() as f32;
        img.resize(720, (img.height() as f32 * ratio) as u32, image::imageops::FilterType::Lanczos3)
    } else {
        img.to_owned()
    };
    // let resized_img = img;

    let end_time = Instant::now();
    println!("Phase 3: {} milliseconds", (end_time - start_time).as_millis());
    
    let mut file = File::create("output.webp").unwrap();

    resized_img.write_to(&mut file, image::ImageOutputFormat::WebP).unwrap();

    let end_time = Instant::now();
    println!("Phase 4: {} milliseconds", (end_time - start_time).as_millis());


    Ok(Created::new("good"))
}