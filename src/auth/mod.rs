use std::pin::Pin;

use actix_web::FromRequest;
use argon2::{
    password_hash::{rand_core::OsRng, Salt, SaltString},
    Argon2, PasswordHasher,
};
use fiole::Wtxn;
use fiole::{codec::FacetJson, Readable};
use uuid::Uuid;

use crate::{
    api::{
        auth::{AdminSignUpRequest, LoginRequest, LoginResponse, UserColor, UserLabel, UserStatus},
        config::Config,
    },
    auth::error::{AdminRegisterError, AuthenticationError, LoginError},
    DbAccessError, MainDatabase, User, UserId, UserMapping,
};

pub mod error;
pub mod middleware;
pub mod token_db;

#[derive(Debug)]
pub struct UserExtractor(pub String);

impl FromRequest for UserExtractor {
    type Error = AuthenticationError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        use actix_web::HttpMessage;
        let mut ext = req.extensions_mut();
        let user = ext.remove::<Self>();
        Box::pin(async move {
            match user {
                Some(user) => Ok(user),
                None => Err(AuthenticationError::InternalCalledOnNonAuthRoute),
            }
        })
    }
}

impl MainDatabase {
    pub const AUTH_KEYSPACE: &str = "auth";

    pub async fn register_admin(
        &self,
        wtxn: &mut Wtxn,
        req: AdminSignUpRequest,
    ) -> Result<User, AdminRegisterError> {
        let now = jiff::Timestamp::now();

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(AdminRegisterError::CouldNotHashPassword)?
            .to_string();
        let mapping = UserMapping {
            password_salt: salt.to_string(),
            password_hash,
            id: UserId(Uuid::now_v7()),
        };

        let mapping_prefix = Self::user_mapping_prefix(&req.email);

        self.auth_db
            .remap_value_type::<FacetJson<UserMapping>>()
            .insert(wtxn, &mapping_prefix, &mapping)
            .map_err(|err| DbAccessError::WritingValue {
                key: req.email.to_string().into(),
                // TODO: This can crash and leak the password hash, we should use the facet pretty print directly
                value: facet_json::to_string_pretty(&mapping).unwrap(),
                db_name: Self::AUTH_KEYSPACE.into(),
                error: Box::new(err),
            })?;

        let user = User {
            id: mapping.id,
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

        self.create_user_db(wtxn, mapping.id, &user).await?;

        self.update_global_config(wtxn, |config| Config {
            is_initialized: true,
            ..config
        })?;

        Ok(user)
    }

    pub fn get_user_mapping(
        &self,
        rtxn: &impl Readable,
        email: String,
    ) -> Result<UserMapping, DbAccessError> {
        let mapping_prefix = Self::user_mapping_prefix(&email);
        self.auth_db
            .remap_value_type::<FacetJson<UserMapping>>()
            .get(rtxn, &mapping_prefix)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::AUTH_KEYSPACE.into(),
                error: Box::new(error),
            })?
            .ok_or_else(|| DbAccessError::UserDoesNotExist { email })
    }

    /// It's the caller job to store the `Uuid` in the `AccessTokenDatabase`.
    pub async fn login(
        &self,
        rtxn: &impl Readable,
        req: LoginRequest,
    ) -> Result<LoginResponse, LoginError> {
        let mapping_prefix = Self::user_mapping_prefix(&req.email);
        let mapping = self
            .auth_db
            .remap_value_type::<FacetJson<UserMapping>>()
            .get(rtxn, &mapping_prefix)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::AUTH_KEYSPACE.into(),
                error: Box::new(error),
            })?
            .unwrap();

        let argon2 = Argon2::default();
        let salt = Salt::from_b64(&mapping.password_salt).map_err(LoginError::InternalSalt)?;
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), salt)
            .map_err(LoginError::CouldNotHashPassword)?
            .to_string();

        if mapping.password_hash != password_hash {
            Err(LoginError::BadUserPassword)
        } else {
            let db = self.get_or_open_user_db(mapping.id).await?;
            let user = db.user(rtxn)?;
            Ok(LoginResponse {
                access_token: Uuid::now_v7().to_string(),
                user_id: mapping.id,
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
