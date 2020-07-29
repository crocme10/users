use juniper::{EmptySubscription, FieldResult, IntoFieldError, RootNode};
use slog::Logger;
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
    /// Returns a list of users
    async fn users(&self, context: &Context) -> FieldResult<users::MultiUsersResponseBody> {
        users::list_users(context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }

    /// Find a user by username
    async fn findUserByUsername(
        &self,
        username: String,
        context: &Context,
    ) -> FieldResult<users::SingleUserResponseBody> {
        users::find_user_by_username(context, &username)
            .await
            .map_err(IntoFieldError::into_field_error)
    }
}

pub struct Mutation;

#[juniper::graphql_object(
    Context = Context
)]
impl Mutation {
    async fn add_user(
        &self,
        user: users::UserRequestBody,
        context: &Context,
    ) -> FieldResult<users::SingleUserResponseBody> {
        users::add_user(user, context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }
}
type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}
