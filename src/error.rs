use actix_web::{ResponseError, http::StatusCode};

use crate::DbAccessError;

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    DbAccessError(#[from] DbAccessError),
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::IoError(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::DbAccessError(_db_access_error) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
