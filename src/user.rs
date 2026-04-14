use fiole::{
    codec::{FacetJson, Str, Unspecified},
    Database, Keyspace, Readable, Wtxn,
};
use fjall::KeyspaceCreateOptions;

use crate::{api::users::me::Preferences, config::SystemConfig, DbAccessError, User};

#[derive(Clone)]
pub struct UserDb {
    main: Keyspace<Str, Unspecified>,
}
impl UserDb {
    pub const USER: &str = "user";
    pub const ASSET_PREFIX: &str = "asset";

    pub fn create(
        db: &Database,
        wtxn: &mut Wtxn,
        id: crate::UserId,
        user: &crate::User,
    ) -> Result<Self, fjall::Error> {
        let user_db = Self::open(db, id)?;

        user_db
            .main
            .remap_value_type::<FacetJson<User>>()
            .insert(wtxn, Self::USER, user)
            .map_err(|err| err.unwrap_fjall())?;
        Ok(user_db)
    }

    pub fn open(db: &Database, id: crate::UserId) -> Result<Self, fjall::Error> {
        let keyspace = db.keyspace(&id.0.to_string(), KeyspaceCreateOptions::default)?;

        Ok(Self { main: keyspace })
    }

    // Can't use the crud macro on user because a user doesn't have a default value

    pub fn user(&self, rtxn: &impl Readable) -> Result<User, DbAccessError> {
        self.main
            .remap_value_type::<FacetJson<User>>()
            .get(rtxn, &Self::USER)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::USER.into(),
                error: Box::new(error),
            })
            .map(|user| user.expect("User MUST contains a user definition"))
    }

    pub fn write_user(&self, wtxn: &mut Wtxn, user: User) -> Result<(), DbAccessError> {
        self.main
            .remap_value_type::<FacetJson<User>>()
            .insert(wtxn, Self::USER, &user)
            .map_err(|error| DbAccessError::ReadingValue {
                db_name: Self::USER.into(),
                error: Box::new(error),
            })?;
        Ok(())
    }

    pub fn update_user(
        &self,
        wtxn: &mut Wtxn,
        update: impl Fn(User) -> User,
    ) -> Result<(), DbAccessError> {
        self.write_user(wtxn, (update)(self.user(wtxn)?))
    }

    pub(crate) fn add_media(
        &self,
        asset: crate::api::assets::AssetUpload,
    ) -> Result<(), DbAccessError> {
        todo!()
    }
}

macro_rules! crud_on {
    ($key:ident, $ty:ty) => {
        paste::paste! {
            impl UserDb {
                pub const [<$key:upper>]: &str = stringify!($key);

                pub fn $key(&self, rtxn: &impl Readable) -> Result<$ty, DbAccessError> {
                    match self
                        .main
                        .remap_value_type::<FacetJson<$ty>>()
                        .get(rtxn, Self::[<$key:upper>])
                        .map_err(|error| DbAccessError::ReadingValue {
                            db_name: Self::[<$key:upper>].into(),
                            error: Box::new(error),
                        })? {
                        Some(pref) => Ok(pref),
                        None => {
                            let pref = $ty::default();
                            Ok(pref)
                        }
                    }
                }

                pub fn [<write_ $key>] (
                    &self,
                    wtxn: &mut Wtxn,
                    preferences: &$ty,
                ) -> Result<(), DbAccessError> {
                    self.main.remap_value_type::<FacetJson<$ty>>().insert(
                        wtxn,
                        Self::[<$key:upper>], preferences
                    )
                    .map_err(|error| {
                        DbAccessError::ReadingValue {
                            db_name: Self::[<$key:upper>].into(),
                            error: Box::new(error),
                        }
                    })?;
                    Ok(())
                }

                pub fn [<update_ $key>] (
                    &self,
                    wtxn: &mut Wtxn,
                    update: impl FnOnce($ty) -> $ty,
                ) -> Result<$ty, DbAccessError> {
                    let pref = self.$key(wtxn)?;
                    let pref = (update)(pref);
                    self.[<write_ $key>](wtxn, &pref)?;
                    Ok(pref)
                }
            }
        }
    };
}

crud_on!(preferences, Preferences);
crud_on!(system_config, SystemConfig);
