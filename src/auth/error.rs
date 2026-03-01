use actix_web::{ResponseError, cookie::ParseError, http::StatusCode};

use crate::DbAccessError;

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Invalid auth")]
    MissingAuthCookie,
    #[error("Malformed cookie: {0}")]
    MalformedCookie(#[from] ParseError),
    #[error("Unknown access token")]
    UnknownAccessToken,
    #[error("While parsing cookie got a duplicate field {0}")]
    DuplicateField(&'static str),
    #[error("While parsing cookie got unexpected field {0}")]
    UnexpectedField(String),
    #[error("While parsing cookie field {0} is missing")]
    MissingField(&'static str),
    #[error("Unexpected `immich_auth_type` type value. Was expecting `password` but got {0}")]
    WrongAuthTypeValue(String),
    #[error("Unexpected `immich_is_authenticated` type value. Was expecting `true` but got {0}")]
    WrongIsAuthenticatedValue(String),
}

impl ResponseError for AuthenticationError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthenticationError::MissingAuthCookie => StatusCode::UNAUTHORIZED,
            AuthenticationError::UnknownAccessToken => StatusCode::UNAUTHORIZED,
            AuthenticationError::DuplicateField(_) => StatusCode::BAD_REQUEST,
            AuthenticationError::WrongAuthTypeValue(_) => StatusCode::BAD_REQUEST,
            AuthenticationError::WrongIsAuthenticatedValue(_) => StatusCode::BAD_REQUEST,
            AuthenticationError::UnexpectedField(_) => StatusCode::BAD_REQUEST,
            AuthenticationError::MissingField(_) => StatusCode::BAD_REQUEST,
            AuthenticationError::MalformedCookie(_) => StatusCode::BAD_REQUEST,
        }
    }
}

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
