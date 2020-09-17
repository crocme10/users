use chrono::{DateTime, Utc};
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

use crate::db::model::*;

/// A user
/// TODO Justify why no password
#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: EntityId,
    pub username: String,
    pub email: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserEntity> for User {
    fn from(entity: UserEntity) -> Self {
        let UserEntity {
            id,
            username,
            email,
            active,
            created_at,
            updated_at,
            ..
        } = entity;

        User {
            id,
            username,
            email,
            active,
            created_at,
            updated_at,
        }
    }
}
