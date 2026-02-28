use argon2::{
    Argon2, PasswordHasher,
    password_hash::{Salt, SaltString, rand_core::OsRng},
};
use uuid::Uuid;

use crate::{
    DbAccessError, MainDatabase, User, UserId,
    api::{
        auth::{AdminSignUpRequest, LoginRequest, LoginResponse, UserColor, UserLabel, UserStatus},
        config::Config,
    },
    auth::error::{AdminRegisterError, LoginError},
};

pub mod error;
pub mod token_db;

impl MainDatabase {
    pub const AUTH_KEYSPACE: &str = "auth";

    pub fn register_admin(&self, req: AdminSignUpRequest) -> Result<User, AdminRegisterError> {
        let now = jiff::Timestamp::now();

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(AdminRegisterError::CouldNotHashPassword)?
            .to_string();
        let response = User {
            password_salt: salt.to_string(),
            password_hash,
            id: UserId(Uuid::now_v7()),
            email: req.email,
            name: req.name,
            profile_image_path: String::new(),
            avatar_color: UserColor::Yellow,
            profile_changed_at: now,
            storage_label: UserLabel::Admin,
            should_change_password: false,
            is_admin: true,
            is_onboarded: false,
            created_at: now,
            deleted_at: None,
            updated_at: now,
            oauth_id: String::new(),
            quota_size_in_bytes: None,
            quota_usage_in_bytes: 0,
            status: UserStatus::Active,
            license: None,
        };

        let json = facet_json::to_string(&response).unwrap();
        self.auth_db
            .insert(response.email.as_bytes(), json)
            .map_err(|err| DbAccessError::WritingValue {
                key: response.id.0.to_string().into(),
                // TODO: This can crash and leak the password, we should use the facet pretty print directly
                value: facet_json::to_string_pretty(&response).unwrap(),
                db_name: Self::AUTH_KEYSPACE.into(),
                error: err,
            })?;
        self.update_global_config(|config| Config {
            is_initialized: true,
            ..config
        })?;

        Ok(response)
    }

    /// It's the caller job to store the `Uuid` in the `AccessTokenDatabase`.
    pub fn login(&self, req: LoginRequest) -> Result<LoginResponse, LoginError> {
        let user = self
            .auth_db
            .get(req.email)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::AUTH_KEYSPACE.into(),
                error,
            })?
            .unwrap();
        let user: User = facet_json::from_slice(&user)?;
        let argon2 = Argon2::default();
        let salt = Salt::from_b64(&user.password_salt).map_err(LoginError::InternalSalt)?;
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), salt)
            .map_err(LoginError::CouldNotHashPassword)?
            .to_string();

        if user.password_hash != password_hash {
            Err(LoginError::BadUserPassword)
        } else {
            Ok(LoginResponse {
                access_token: Uuid::now_v7(),
                user_id: user.id,
                user_email: user.email,
                name: user.name,
                is_admin: user.is_admin,
                profile_image_path: user.profile_image_path,
                should_change_password: user.should_change_password,
                is_onboarded: user.is_onboarded,
            })
        }
    }
}
