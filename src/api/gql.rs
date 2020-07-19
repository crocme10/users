use juniper::{EmptySubscription, FieldResult, IntoFieldError, RootNode};
use slog::Logger;
use std::collections::HashMap;

use super::user;

#[derive(Debug, Clone)]
pub struct Context {
    pub logger: Logger,
    pub users: HashMap<String, String>,
}

impl juniper::Context for Context {}

pub struct Query;

#[juniper::graphql_object(
    Context = Context
)]
impl Query {
    /// Return a list of all environments
    async fn users(&self, context: &Context) -> FieldResult<user::MultiUsersResponseBody> {
        user::list_users(context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }
}

pub struct Mutation;

#[juniper::graphql_object(
    Context = Context
)]
impl Mutation {
    /// Return a list of all environments
    async fn addUuser(
        username: String,
        email: String,
        context: &Context,
    ) -> FieldResult<user::SingleUserResponseBody> {
        user::add_user(username, email, context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }
}

type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}
