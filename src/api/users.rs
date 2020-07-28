use futures::TryFutureExt;
use juniper::{GraphQLInputObject, GraphQLObject};
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
/// It is optional, since we may be looking for a user which
/// does not match the query criteria.
#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct SingleUserResponseBody {
    pub user: Option<User>,
}

impl From<User> for SingleUserResponseBody {
    fn from(user: User) -> Self {
        Self { user: Some(user) }
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

/// The query body for creating a user
#[derive(Debug, Serialize, Deserialize, GraphQLInputObject)]
pub struct UserRequestBody {
    pub username: String,
    pub email: String,
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
                msg: "could not initiate transaction",
            })?;

        let entities = tx.get_all_users().await.context(error::DBProvideError {
            msg: "Could not get all them users",
        })?;

        let users = entities
            .into_iter()
            .map(|ent| User::from(ent))
            .collect::<Vec<_>>();

        tx.commit().await.context(error::DBError {
            msg: "could not commit transaction",
        })?;

        Ok(MultiUsersResponseBody::from(users))
    }
    .await
}

/// Create a new user.
pub async fn add_user(
    user_request: UserRequestBody,
    context: &Context,
) -> Result<SingleUserResponseBody, error::Error> {
    async move {
        let UserRequestBody { username, email } = user_request;

        let state = &context.pool;

        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .context(error::DBError {
                msg: "could not initiate transaction",
            })?;

        let entity = tx
            .create_user(&username, &email)
            .await
            .context(error::DBProvideError {
                msg: "Could not create user",
            })?;

        let user = User::from(entity);

        tx.commit().await.context(error::DBError {
            msg: "could not retrieve indexes",
        })?;

        Ok(SingleUserResponseBody::from(user))
    }
    .await
}

/// Retrieve a single user given its username
pub async fn find_user_by_username(
    context: &Context,
    username: &str,
) -> Result<SingleUserResponseBody, error::Error> {
    async move {
        let state = &context.pool;

        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .context(error::DBError {
                msg: "could not initiate transaction",
            })?;

        let entity = tx
            .get_user_by_username(username)
            .await
            .context(error::DBProvideError {
                msg: "Could not get user by username",
            })?;

        let user = User::from(entity);

        tx.commit().await.context(error::DBError {
            msg: "could not commit transaction",
        })?;

        Ok(SingleUserResponseBody::from(user))
    }
    .await
}
