use rocket::{response::status::Custom, serde::json::Json, http::Status};
use serde::Serialize;

pub type CustomError = Custom<Json<ErrorInfo>>;

#[derive(Serialize)]
pub struct ErrorInfo {
    success: bool,
    message: String,
}

impl ErrorInfo {
    pub fn new(message: String) -> Self {
        Self { success: false, message }
    }
}

pub fn handle_error(e: diesel::result::Error) -> CustomError {
    use diesel::result::DatabaseErrorKind::*;
    use diesel::result::Error::*;

    match e {
        InvalidCString(_) => Custom(Status::UnprocessableEntity, Json(ErrorInfo::new("invalid_c_string".to_string()))),
        DatabaseError(kind, _info) => match kind {
            UniqueViolation => Custom(Status::Conflict, Json(ErrorInfo::new("unique_violation".to_string()))),
            ForeignKeyViolation => Custom(Status::UnprocessableEntity, Json(ErrorInfo::new("foreign_key_violation".to_string()))),
            NotNullViolation => Custom(Status::UnprocessableEntity, Json(ErrorInfo::new("not_null_violation".to_string()))),
            CheckViolation => Custom(Status::UnprocessableEntity, Json(ErrorInfo::new("check_violation".to_string()))),
            ClosedConnection => Custom(Status::ServiceUnavailable, Json(ErrorInfo::new("closed_connection".to_string()))),
            _ => Custom(Status::InternalServerError, Json(ErrorInfo::new("internal_server_error".to_string()))),
        },
        NotFound => Custom(Status::NotFound, Json(ErrorInfo::new("not_found".to_string()))),
        DeserializationError(_) => Custom(Status::UnprocessableEntity, Json(ErrorInfo::new("deserialization_error".to_string()))),
        _ => Custom(Status::InternalServerError, Json(ErrorInfo::new("internal_server_error".to_string()))),
    }
}