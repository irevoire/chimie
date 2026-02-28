use actix_web::{ResponseError, http::StatusCode};

use crate::{
    DbAccessError,
    auth::error::{AdminRegisterError, AuthenticationError, LoginError},
};

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    DbAccess(#[from] DbAccessError),
    #[error(transparent)]
    Login(#[from] LoginError),
    #[error(transparent)]
    Register(#[from] AdminRegisterError),
    #[error(transparent)]
    Auth(#[from] AuthenticationError),
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::Io(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::DbAccess(_db_access_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::Login(login_error) => login_error.status_code(),
            HttpError::Register(admin_register_error) => admin_register_error.status_code(),
            HttpError::Auth(authentication_error) => authentication_error.status_code(),
        }
    }
}
