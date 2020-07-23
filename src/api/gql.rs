use juniper::{EmptyMutation, EmptySubscription, FieldResult, IntoFieldError, RootNode};
use slog::Logger;
// use sqlx::pool::PoolConnection;
// use snafu::ResultExt;
// use crate::db::model::ProvideData;
// use crate::db::Db;
use sqlx::postgres::PgPool;

use super::users;

#[derive(Debug, Clone)]
pub struct Context {
    pub logger: Logger,
    pub pool: PgPool,
}

impl juniper::Context for Context {}

pub struct Query;

#[juniper::graphql_object(
    Context = Context
)]
impl Query {
    /// Return a list of all environments
    async fn users(&self, context: &Context) -> FieldResult<users::MultiUsersResponseBody> {
        users::list_users(context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}
