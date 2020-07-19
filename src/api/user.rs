use chrono::prelude::*;
// use futures::future::TryFutureExt;
use futures::stream::{self, TryStreamExt};
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use super::gql::Context;
use crate::error;

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

#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
pub struct User {
    pub username: String,
    pub email: String,
    #[serde(default = "now")]
    pub created_at: DateTime<Utc>,
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

pub async fn list_users(context: &Context) -> Result<MultiUsersResponseBody, error::Error> {
    let users = stream::iter(context.users.iter().map(|user| Ok(user)))
        .try_fold(Vec::new(), |mut acc, (username, email)| async move {
            acc.push(User {
                username: username.into(),
                email: email.into(),
                created_at: Utc::now(),
            });
            Ok(acc)
        })
        .await?;
    Ok(users.into())
}

pub async fn add_user(
    username: String,
    email: String,
    context: &mut Context,
) -> Result<SingleUserResponseBody, error::Error> {
    // In this simplified version, the database is just a hashmap of username -> email.
    match context.users.insert(username.clone(), email.clone()) {
        None => {
            let user = User {
                username,
                email,
                created_at: Utc::now(),
            };
            Ok(user.into())
        }
        Some(_) => Err(error::Error::MiscError {
            msg: String::from("Duplicate Entry"),
        }),
    }
}
