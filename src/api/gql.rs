use juniper::GraphQLObject;
use juniper::{EmptySubscription, FieldResult, IntoFieldError, RootNode};
use serde::{Deserialize, Serialize};
use slog::info;

use super::users;
use crate::error;
//use crate::state::jwt::Jwt;
use crate::state::state::State;

/// A test content
#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct ContentResponseBody {
    pub content: String,
}

impl From<String> for ContentResponseBody {
    fn from(content: String) -> Self {
        Self { content }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub state: State,
    pub token: Option<String>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn is_authenticated(&self) -> bool {
        info!(self.state.logger, "auth check: token: {:?}", self.token);
        match self
            .token
            .as_deref()
            .ok_or(error::Error::MiscError {
                msg: String::from("Unauthenticated Access"),
            })
            .and_then(|tok| self.state.jwt.decode(tok))
        {
            Ok(claimset) => {
                let options: biscuit::Validation<biscuit::TemporalOptions> =
                    biscuit::Validation::Validate(Default::default());
                match claimset.registered.validate_exp(options) {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
    pub fn is_admin(&self) -> bool {
        match self
            .token
            .as_deref()
            .ok_or(error::Error::MiscError {
                msg: String::from("Unauthenticated Access"),
            })
            .and_then(|tok| self.state.jwt.decode(tok))
        {
            Ok(_) => false,
            Err(_) => false,
        }
    }
}
pub struct Query;

#[juniper::graphql_object(
    Context = Context
)]
impl Query {
    /// Returns a list of users
    async fn users(&self, context: &Context) -> FieldResult<users::MultiUsersResponseBody> {
        if let Some(token) = &context.token {
            info!(context.state.logger, "auth token: {}", token);
        }
        users::list_users(context)
            .await
            .map_err(IntoFieldError::into_field_error)
            .into()
    }

    /// Returns content for all
    /// This content is for anyone, and there are no checks
    async fn content_for_all(&self, context: &Context) -> FieldResult<ContentResponseBody> {
        if let Some(token) = &context.token {
            info!(context.state.logger, "all auth token: {}", token);
        }
        let res = ContentResponseBody::from(String::from("Hello, all"));
        let res: Result<ContentResponseBody, error::Error> = Ok(res);
        res.map_err(IntoFieldError::into_field_error)
    }

    /// Returns content for user
    /// This content is for registered user.
    async fn content_for_user(&self, context: &Context) -> FieldResult<ContentResponseBody> {
        if !context.is_authenticated() {
            return Err(IntoFieldError::into_field_error(error::Error::MiscError {
                msg: String::from("Unauthenticated Access"),
            }));
        }
        let res = ContentResponseBody::from(String::from("Hello, user"));
        let res: Result<ContentResponseBody, error::Error> = Ok(res);
        res.map_err(IntoFieldError::into_field_error)
    }

    /// Returns content for moderator
    /// This content is for registered user.
    async fn content_for_admin(&self, context: &Context) -> FieldResult<ContentResponseBody> {
        if !context.is_admin() {
            return Err(IntoFieldError::into_field_error(error::Error::MiscError {
                msg: String::from("Unauthenticated Access"),
            }));
        }
        let res = ContentResponseBody::from(String::from("Hello, admin"));
        let res: Result<ContentResponseBody, error::Error> = Ok(res);
        res.map_err(IntoFieldError::into_field_error)
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

    async fn register_user(
        &self,
        user: users::UserRequestBody,
        context: &Context,
    ) -> FieldResult<users::SingleUserResponseBody> {
        users::register_user(user, context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }

    async fn login_user(
        &self,
        credentials: users::CredentialsRequestBody,
        context: &Context,
    ) -> FieldResult<users::AuthenticatedUserResponseBody> {
        users::login_user(credentials, context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }
}
type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}
