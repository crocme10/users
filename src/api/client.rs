use futures::future::TryFutureExt;
use snafu::futures::try_future::TryFutureExt as SnafuTryFutureExt;
use snafu::ResultExt;

use super::users::{MultiUsersResponseBody, SingleUserResponseBody, UserRequestBody};
use crate::error;
use crate::utils::{construct_headers, get_service_url};

// Request a list of users.
// TODO We rely on a helper function `get_service_url` to identify the target service
// but this is probably not the best solution. Maybe the service's url needs to be
// passed as another function argument.
pub async fn list_users() -> Result<MultiUsersResponseBody, error::Error> {
    let data = get_graphql_str_list_users();
    let url = get_service_url();
    let client = reqwest::Client::new();
    client
        .post(&url)
        .headers(construct_headers())
        .body(data)
        .send()
        .context(error::ReqwestError {
            msg: String::from("Could not query users"),
        })
        .and_then(|resp| {
            resp.json::<serde_json::Value>()
                .context(error::ReqwestError {
                    msg: String::from("Could not deserialize MultiUsersResponseBody"),
                })
        })
        .and_then(|json| {
            // This json object can be either { data: { users: { } } } if the call was successful,
            // or { data: null, error: [ ] } if the call was not successful,
            // FIXME Lots of unwrap in the following code, also I don't extract the 'message' part
            // of the error.
            async move {
                let data = &json["data"];
                if data.is_null() {
                    if let Some(errors) = json.get("errors") {
                        let errors = errors.clone();
                        let errors = errors.as_array().expect("errors is an array");
                        let error = &errors.first().expect("at least one error");
                        let msg = error
                            .get("extensions")
                            .expect("error to have an extension field")
                            .get("internal_error")
                            .expect("extension to have an internal_error field");
                        return Err(error::Error::MiscError {
                            msg: format!("Error while requesting users: {}", msg),
                        });
                    } else {
                        return Err(error::Error::MiscError {
                            msg: String::from("Data is null, and there are no errors."),
                        });
                    }
                } else {
                    if let Some(users) = data.get("users") {
                        let users = users.clone();
                        serde_json::from_value(users).context(error::JSONError {
                            msg: String::from("Could not deserialize users"),
                        })
                    } else {
                        Err(error::Error::MiscError {
                            msg: String::from("Data is not null, and there are no users."),
                        })
                    }
                }
            }
        })
        .await
}

pub async fn add_user(user: UserRequestBody) -> Result<SingleUserResponseBody, error::Error> {
    let data = get_graphql_str_add_user(user);
    let url = get_service_url();
    let client = reqwest::Client::new();
    client
        .post(&url)
        .headers(construct_headers())
        .body(data)
        .send()
        .context(error::ReqwestError {
            msg: String::from("Could not request SingleUserResponseBody"),
        })
        .and_then(|resp| {
            resp.json::<serde_json::Value>()
                .context(error::ReqwestError {
                    msg: String::from("Could not deserialize SingleUserResponseBody"),
                })
        })
        .and_then(|json| {
            async move {
                // This JSON contains two fields, data, and errors.
                // So we test if data is null,
                //   in which case we return the first error in the errors array,
                // otherwise
                //   we return the expected singleuserresponse
                if json["data"].is_null() {
                    let errors = json["errors"].as_array().expect("errors");
                    let error = &errors.first().expect("at least one error");
                    Err(error::Error::MiscError {
                        msg: format!("{}", error),
                    })
                } else {
                    let res = &json["data"]["addUser"];
                    let res = res.clone();
                    serde_json::from_value(res).context(error::JSONError {
                        msg: String::from("Can not retrieve singleuserresponse"),
                    })
                }
            }
            // .context(error::JSONError {
            //     msg: String::from("Could not deserialize MultiUsersResponseBody"),
            // })
        })
        .await
}

pub async fn find_user_by_username(
    username: String,
) -> Result<SingleUserResponseBody, error::Error> {
    let data = get_graphql_str_find_user(&username);
    let url = get_service_url();
    let client = reqwest::Client::new();
    client
        .post(&url)
        .headers(construct_headers())
        .body(data)
        .send()
        .context(error::ReqwestError {
            msg: String::from("Could not request SingleUserResponseBody"),
        })
        .and_then(|resp| {
            // async move {
            //     let txt = resp.text().await.unwrap();
            //     println!("text: {}", txt);
            //     let err: Result<SingleUserResponseBody, _> = Err(error::Error::MiscError {
            //         msg: format!("{}", "foo"),
            //     });
            //     err
            // }

            resp.json::<serde_json::Value>()
                .context(error::ReqwestError {
                    msg: String::from("Could not deserialize MultiUsersResponseBody"),
                })
        })
        .and_then(|json| {
            async move {
                // This JSON contains two fields, data, and errors.
                // So we test if data is null,
                //   in which case we return the first error in the errors array,
                // otherwise
                //   we return the expected singleuserresponse
                if json["data"].is_null() {
                    let errors = json["errors"].as_array().expect("errors");
                    let error = &errors.first().expect("at least one error");
                    Err(error::Error::MiscError {
                        msg: format!("{}", error),
                    })
                } else {
                    let res = &json["data"]["findUserByUsername"];
                    let res = res.clone();
                    serde_json::from_value(res).context(error::JSONError {
                        msg: String::from("Can not retrieve singleuserresponse"),
                    })
                }
            }
        })
        .await
}

// This is a helper function which generates the GraphQL query for listing users
pub fn get_graphql_str_list_users() -> String {
    String::from("{ \"query\": \"{ users { users { id, username, email, roles, active, createdAt, updatedAt }, usersCount } }\" }")
}

// This is a helper function which generates the GraphQL query for adding a user.
pub fn get_graphql_str_add_user(user: UserRequestBody) -> String {
    let query = r#" "mutation addUser($user: UserRequestBody!) { addUser(user: $user) { user { id, username, email, roles, active, createdAt, updatedAt } } }" "#;
    let variables = serde_json::to_string(&user).unwrap();
    format!(
        r#"{{ "query": {query}, "variables": {{ "user": {variables} }} }}"#,
        query = query,
        variables = variables
    )
}

// This is a helper function which generates the GraphQL query for finding a user.
pub fn get_graphql_str_find_user(username: &str) -> String {
    let query = r#" "query findUser($username: String!) { findUserByUsername(username: $username) { user { id, username, email, roles, active, createdAt, updatedAt } } }" "#;
    let variables = serde_json::to_string(username).unwrap();
    format!(
        r#"{{ "query": {query}, "variables": {{ "username": {variables} }} }}"#,
        query = query,
        variables = variables
    )
}

pub mod blocking {
    use crate::api::users::{MultiUsersResponseBody, SingleUserResponseBody, UserRequestBody};
    use crate::error;
    pub fn list_users() -> Result<MultiUsersResponseBody, error::Error> {
        // We use the Client API, which is async, so we need to wrap it around some
        // tokio machinery to spin the async code in a thread, and wait for the result.
        let handle = tokio::runtime::Handle::current();
        let th = std::thread::spawn(move || {
            match handle.block_on(async { super::list_users().await }) {
                Ok(m) => Ok(m),
                Err(err) => Err(err),
            }
        });
        th.join().unwrap()
    }
    pub fn add_user(user: UserRequestBody) -> Result<SingleUserResponseBody, error::Error> {
        // We use the Client API, which is async, so we need to wrap it around some
        // tokio machinery to spin the async code in a thread, and wait for the result.
        // FIXME We're not extracting the error properly
        let handle = tokio::runtime::Handle::current();
        let th = std::thread::spawn(move || handle.block_on(async { super::add_user(user).await }));
        th.join().unwrap()
    }
    pub fn find_user_by_username(username: String) -> Result<SingleUserResponseBody, error::Error> {
        // We use the Client API, which is async, so we need to wrap it around some
        // tokio machinery to spin the async code in a thread, and wait for the result.
        let handle = tokio::runtime::Handle::current();
        let th = std::thread::spawn(move || {
            handle.block_on(async { super::find_user_by_username(username).await })
        });
        th.join().unwrap()
    }
}
