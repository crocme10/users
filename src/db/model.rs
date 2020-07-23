use async_trait::async_trait;
use chrono::{DateTime, Utc};
use snafu::Snafu;
use uuid::Uuid;

pub type EntityId = Uuid;

/// A user registered with the application (ie, stored in DB)
pub struct UserEntity {
    pub id: EntityId,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait ProvideData {
    async fn create_user(&mut self, username: &str, email: &str) -> ProvideResult<UserEntity>;

    async fn get_all_users(&mut self) -> ProvideResult<Vec<UserEntity>>;
}

pub type ProvideResult<T> = Result<T, ProvideError>;

/// An error returned by a provider
#[derive(Debug, Snafu)]
pub enum ProvideError {
    /// The requested entity does not exist
    #[snafu(display("Entity does not exist"))]
    #[snafu(visibility(pub))]
    NotFound,

    /// The operation violates a uniqueness constraint
    #[snafu(display("Operation violates uniqueness constraint: {}", details))]
    #[snafu(visibility(pub))]
    UniqueViolation { details: String },

    /// The requested operation violates the data model
    #[snafu(display("Operation violates model: {}", details))]
    #[snafu(visibility(pub))]
    ModelViolation { details: String },

    /// The requested operation violates the data model
    #[snafu(display("UnHandled Error: {}", source))]
    #[snafu(visibility(pub))]
    UnHandledError { source: sqlx::Error },
}

impl From<sqlx::Error> for ProvideError {
    /// Convert a SQLx error into a provider error
    ///
    /// For Database errors we attempt to downcast
    ///
    /// FIXME(RFC): I have no idea if this is sane
    fn from(e: sqlx::Error) -> Self {
        // TODO See if we can we actually do some logging here?
        // log::debug!("sqlx returned err -- {:#?}", &e);
        match e {
            sqlx::Error::RowNotFound => ProvideError::NotFound,
            sqlx::Error::Database(db_err) => {
                #[cfg(feature = "postgres")]
                {
                    if let Some(pg_err) = db_err.try_downcast_ref::<sqlx::postgres::PgError>() {
                        if let Ok(provide_err) = ProvideError::try_from(pg_err) {
                            return provide_err;
                        }
                    }
                }

                #[cfg(feature = "sqlite")]
                {
                    if let Some(sqlite_err) = db_err.try_downcast_ref::<sqlx::sqlite::SqliteError>()
                    {
                        if let Ok(provide_err) = ProvideError::try_from(sqlite_err) {
                            return provide_err;
                        }
                    }
                }

                ProvideError::UnHandledError {
                    source: sqlx::Error::Database(db_err),
                }
            }
            _ => ProvideError::UnHandledError { source: e },
        }
    }
}
