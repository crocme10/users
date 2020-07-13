use juniper::{EmptyMutation, EmptySubscription, FieldResult, IntoFieldError, RootNode};
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

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}
