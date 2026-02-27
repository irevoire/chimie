use actix_web::{ResponseError, http::StatusCode};

use crate::{DbAccessError, LoginError};

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    DbAccess(#[from] DbAccessError),
    #[error(transparent)]
    Login(#[from] LoginError),
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::Io(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::DbAccess(_db_access_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::Login(login_error) => match login_error {
                LoginError::DbAccessError(_db_access_error) => StatusCode::INTERNAL_SERVER_ERROR,
                LoginError::BadUserPassword => StatusCode::UNAUTHORIZED,
                LoginError::InternalDeserializationError(_deserialize_error) => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                LoginError::InternalSaltError(_error) => StatusCode::INTERNAL_SERVER_ERROR,
                LoginError::CouldNotHashPassword(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}
