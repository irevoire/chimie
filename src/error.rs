use actix_web::{http::StatusCode, ResponseError};

use crate::{
    auth::error::{AdminRegisterError, AuthenticationError, LoginError},
    DbAccessError,
};

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ActixWeb(#[from] actix_web::Error),
    #[error(transparent)]
    Fjall(#[from] fjall::Error),
    #[error(transparent)]
    DbAccess(#[from] DbAccessError),
    #[error(transparent)]
    Login(#[from] LoginError),
    #[error(transparent)]
    Register(#[from] AdminRegisterError),
    #[error(transparent)]
    Auth(#[from] AuthenticationError),
    #[error("The non admin user with email `{user}` tried to finalize the system onboarding.")]
    NonAdminUserTriedToFinalizeSystemOnboarding { user: String },
    #[error(transparent)]
    CommitFailed(#[from] fjall::Conflict),
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::Io(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::DbAccess(_db_access_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::Login(login_error) => login_error.status_code(),
            HttpError::Register(admin_register_error) => admin_register_error.status_code(),
            HttpError::Auth(authentication_error) => authentication_error.status_code(),
            HttpError::Fjall(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::ActixWeb(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::CommitFailed(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::NonAdminUserTriedToFinalizeSystemOnboarding { .. } => {
                StatusCode::UNAUTHORIZED
            }
        }
    }
}
