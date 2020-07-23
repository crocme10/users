use chrono::{DateTime, Utc};
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

use crate::db::model::*;

/// A user
#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
// pub(in crate::api) struct User {
pub struct User {
    pub id: EntityId,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserEntity> for User {
    fn from(entity: UserEntity) -> Self {
        let UserEntity {
            id,
            username,
            email,
            created_at,
            updated_at,
            ..
        } = entity;

        User {
            id,
            username,
            email,
            created_at,
            updated_at,
        }
    }
}
