use async_trait::async_trait;
use chrono::{DateTime, Utc};
// use snafu::ResultExt;
use sqlx::error::DatabaseError;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgError, PgQueryAs, PgRow};
use sqlx::row::{FromRow, Row};
use sqlx::{PgConnection, PgPool};
use std::convert::TryFrom;

use super::model;
use super::Db;
// use crate::error;

/// A user registered with the application (Postgres version)
pub struct UserEntity {
    pub id: model::EntityId,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'c> FromRow<'c, PgRow<'c>> for UserEntity {
    fn from_row(row: &PgRow<'c>) -> Result<Self, sqlx::Error> {
        Ok(UserEntity {
            id: row.get(0),
            username: row.get(1),
            email: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
        })
    }
}

impl From<UserEntity> for model::UserEntity {
    fn from(pg: UserEntity) -> Self {
        let UserEntity {
            id,
            username,
            email,
            created_at,
            updated_at,
        } = pg;

        model::UserEntity {
            id,
            username,
            email,
            created_at,
            updated_at,
        }
    }
}

/// Open a connection to a database
pub async fn connect(db_url: &str) -> sqlx::Result<PgPool> {
    let pool = PgPool::new(db_url).await?;
    Ok(pool)
}

impl TryFrom<&PgError> for model::ProvideError {
    type Error = ();

    /// Attempt to convert a Postgres error into a generic ProvideError
    ///
    /// Unexpected cases will be bounced back to the caller for handling
    ///
    /// * [Postgres Error Codes](https://www.postgresql.org/docs/current/errcodes-appendix.html)
    fn try_from(pg_err: &PgError) -> Result<Self, Self::Error> {
        let provider_err = match pg_err.code().unwrap() {
            "23505" => model::ProvideError::UniqueViolation {
                details: pg_err.details().unwrap().to_owned(),
            },
            code if code.starts_with("23") => model::ProvideError::ModelViolation {
                details: pg_err.message().to_owned(),
            },
            _ => return Err(()),
        };

        Ok(provider_err)
    }
}

#[async_trait]
impl Db for PgPool {
    type Conn = PoolConnection<PgConnection>;

    async fn conn(&self) -> Result<Self::Conn, sqlx::Error> {
        self.acquire().await
    }
}

#[async_trait]
impl model::ProvideData for PgConnection {
    async fn create_user(
        &mut self,
        username: &str,
        email: &str,
    ) -> model::ProvideResult<model::UserEntity> {
        let user: UserEntity = sqlx::query_as(
            r#"
INSERT INTO users ( username, email )
VALUES ( $1, $2 )
RETURNING *
        "#,
        )
        .bind(username)
        .bind(email)
        .fetch_one(self)
        .await?;

        Ok(user.into())
    }

    async fn get_all_users(&mut self) -> model::ProvideResult<Vec<model::UserEntity>> {
        let users: Vec<UserEntity> = sqlx::query_as(
            r#"
SELECT *
FROM users
ORDER BY created_at
            "#,
        )
        .fetch_all(self)
        .await?;

        let users = users
            .into_iter()
            .map(|u| model::UserEntity::from(u))
            .collect::<Vec<_>>();

        Ok(users)
    }
}