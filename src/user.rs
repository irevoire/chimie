use fjall::KeyspaceCreateOptions;
use fjall_typed::{
    codec::{FacetJson, Str, Unspecified},
    OptimisticTxKeyspace, OptimisticWriteTx, TypedReadable,
};

use crate::{api::users::me::Preferences, config::SystemConfig, DbAccessError, User};

#[derive(Clone)]
pub struct UserDb {
    main: OptimisticTxKeyspace<'static, Str, Unspecified>,
}
impl UserDb {
    pub const USER: &str = "user";
    pub const ASSET_PREFIX: &str = "asset";

    pub fn create(
        db: &fjall::OptimisticTxDatabase,
        id: crate::UserId,
        user: &crate::User,
    ) -> Result<Self, fjall::Error> {
        let user_db = Self::open(db, id)?;

        user_db
            .main
            .remap_value::<FacetJson<User>>()
            .insert(Self::USER, user)
            .map_err(|err| err.unwrap_fjall())?;
        Ok(user_db)
    }

    pub fn open(db: &fjall::OptimisticTxDatabase, id: crate::UserId) -> Result<Self, fjall::Error> {
        let keyspace = db.keyspace(&id.0.to_string(), KeyspaceCreateOptions::default)?;
        let keyspace: OptimisticTxKeyspace<'static, Str, Unspecified> =
            OptimisticTxKeyspace::new(keyspace);

        Ok(Self { main: keyspace })
    }

    // Can't use the crud macro on user because a user doesn't have a default value

    pub fn user(&self, rtxn: &impl TypedReadable) -> Result<User, DbAccessError> {
        TypedReadable::get(
            rtxn,
            self.main.remap_value::<FacetJson<User>>().as_keyspace(),
            &Self::USER,
        )
        .map_err(|error| DbAccessError::ReadingValue {
            db_name: Self::USER.into(),
            error: Box::new(error),
        })
        .map(|user| user.expect("User MUST contains a user definition"))
    }

    pub fn write_user(
        &self,
        wtxn: &mut OptimisticWriteTx,
        user: User,
    ) -> Result<(), DbAccessError> {
        wtxn.insert(
            self.main.remap_value::<FacetJson<User>>().as_keyspace(),
            Self::USER,
            &user,
        )
        .map_err(|error| DbAccessError::ReadingValue {
            db_name: Self::USER.into(),
            error: Box::new(error),
        })?;
        Ok(())
    }

    pub fn update_user(
        &self,
        wtxn: &mut OptimisticWriteTx,
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

                pub fn $key(&self, rtxn: &impl TypedReadable) -> Result<$ty, DbAccessError> {
                    match TypedReadable::get(
                            rtxn,
                            self
                                .main
                                .remap_value::<FacetJson<$ty>>()
                                .as_keyspace(),
                            Self::[<$key:upper>],
                        )
                        .map_err(|error| DbAccessError::ReadingValue {
                            db_name: Self::[<$key:upper>].into(),
                            error: Box::new(error),
                        })? {
                        Some(pref) => Ok(pref),
                        None => {
                            let pref = $ty::default();
                            self.main
                                .remap_value::<FacetJson<$ty>>()
                                .insert(Self::[<$key:upper>], &pref)
                                .map_err(|error| DbAccessError::WritingValue {
                                    key: Self::[<$key:upper>].into(),
                                    value: format!("{pref:?}"),
                                    db_name: "".into(),
                                    error: Box::new(error),
                                })?;
                            Ok(pref)
                        }
                    }
                }

                pub fn [<write_ $key>] (
                    &self,
                    wtxn: &mut OptimisticWriteTx,
                    preferences: &$ty,
                ) -> Result<(), DbAccessError> {
                    wtxn.insert(
                        self.main.remap_value::<FacetJson<$ty>>().as_keyspace(),
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
                    wtxn: &mut OptimisticWriteTx,
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
