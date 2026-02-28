use actix_web::{ResponseError, http::StatusCode};

use crate::DbAccessError;

#[derive(Debug, thiserror::Error)]
pub enum AdminRegisterError {
    #[error(transparent)]
    DbAccess(#[from] DbAccessError),
    #[error("Bad user/password")]
    BadUserPassword,
    #[error("Couldn't deserialize malformed user: {0}")]
    InternalDeserialization(#[from] facet_json::DeserializeError),
    #[error("Could not hash password but why???: {0}")]
    CouldNotHashPassword(argon2::password_hash::Error),
}

impl ResponseError for AdminRegisterError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AdminRegisterError::DbAccess(_db_access_error) => StatusCode::INTERNAL_SERVER_ERROR,
            AdminRegisterError::BadUserPassword => StatusCode::UNAUTHORIZED,
            AdminRegisterError::InternalDeserialization(_deserialize_error) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AdminRegisterError::CouldNotHashPassword(_error) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error(transparent)]
    DbAccess(#[from] DbAccessError),
    #[error("Bad user/password")]
    BadUserPassword,
    #[error("Couldn't deserialize malformed user: {0}")]
    InternalDeserialization(#[from] facet_json::DeserializeError),
    #[error("Bad salt in database: {0}")]
    InternalSalt(argon2::password_hash::Error),
    #[error("Could not hash password but why???: {0}")]
    CouldNotHashPassword(argon2::password_hash::Error),
}

impl ResponseError for LoginError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            LoginError::DbAccess(_db_access_error) => StatusCode::INTERNAL_SERVER_ERROR,
            LoginError::BadUserPassword => StatusCode::UNAUTHORIZED,
            LoginError::InternalDeserialization(_deserialize_error) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            LoginError::InternalSalt(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            LoginError::CouldNotHashPassword(_error) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
