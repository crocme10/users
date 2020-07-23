use futures::TryFutureExt;
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
// use slog::{debug, info};
use snafu::ResultExt;
use sqlx::Connection;
use std::convert::TryFrom;

use crate::api::gql::Context;
use crate::api::model::*;
use crate::db::model::ProvideData;
use crate::db::Db;
use crate::error;
// use crate::fsm;

/// The response body for single user
#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct SingleUserResponseBody {
    pub user: User,
}

impl From<User> for SingleUserResponseBody {
    fn from(user: User) -> Self {
        Self { user }
    }
}

/// The response body for multiple users
#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct MultiUsersResponseBody {
    pub users: Vec<User>,
    pub users_count: i32,
}

impl From<Vec<User>> for MultiUsersResponseBody {
    fn from(users: Vec<User>) -> Self {
        let users_count = i32::try_from(users.len()).unwrap();
        Self { users, users_count }
    }
}

/// Retrieve all users
pub async fn list_users(context: &Context) -> Result<MultiUsersResponseBody, error::Error> {
    async move {
        let state = &context.pool;

        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .context(error::DBError {
                msg: "could not retrieve indexes",
            })?;

        let entities = tx.get_all_users().await.context(error::DBProvideError {
            msg: "Could not get all them indexes",
        })?;

        let users = entities
            .into_iter()
            .map(|ent| User::from(ent))
            .collect::<Vec<_>>();

        tx.commit().await.context(error::DBError {
            msg: "could not retrieve indexes",
        })?;

        Ok(MultiUsersResponseBody::from(users))
    }
    .await
}
