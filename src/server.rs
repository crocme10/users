use clap::ArgMatches;
use slog::{info, o, Logger};
use snafu::ResultExt;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use warp::{self, http, Filter};

use users::api::{gql, user::User};
use users::error;
use users::settings::Settings;

pub async fn run<'a>(matches: &ArgMatches<'a>, logger: Logger) -> Result<(), error::Error> {
    let settings = Settings::new(matches)?;
    let s2 = settings.clone();

    let clogger = logger.new(
        o!("host" => s2.service.host, "port" => s2.service.port, "database" => s2.database.url),
    );

    let users = tokio::fs::read_to_string("users.json")
        .await
        .context(error::TokioIOError {
            msg: String::from("Could not open users.json"),
        })?;
    let users: Vec<User> = serde_json::from_str(&users).context(error::JSONError {
        msg: String::from("Could not deserialize users.json content"),
    })?;
    let users: HashMap<String, String> = users.into_iter().map(|u| (u.username, u.email)).collect();
    info!(clogger, "Initialized users");

    run_server(settings, clogger, users).await
}

async fn run_server(
    settings: Settings,
    logger: Logger,
    users: HashMap<String, String>,
) -> Result<(), error::Error> {
    let logger1 = logger.clone();
    let users1 = users.clone();
    let state = warp::any().map(move || gql::Context {
        logger: logger1.clone(),
        users: users1.clone(),
    });

    let playground = warp::get()
        .and(warp::path("playground"))
        .and(playground_filter("/graphql", Some("/subscriptions")));

    let graphql_filter = juniper_warp::make_graphql_filter(gql::schema(), state.boxed());

    let graphql = warp::path!("graphql").and(graphql_filter);

    let routes = playground.or(graphql);

    let host = settings.service.host;
    let port = settings.service.port;
    let addr = (host.as_str(), port);

    let addr = addr
        .to_socket_addrs()
        .context(error::IOError {
            msg: String::from("To Sock Addr"),
        })?
        .next()
        .ok_or(error::Error::MiscError {
            msg: String::from("Cannot resolve addr"),
        })?;

    info!(
        logger.clone(),
        "Serving Users on {}:{}",
        addr.ip(),
        addr.port()
    );
    warp::serve(routes).run(addr).await;

    Ok(())
}

/// Create a filter that replies with an HTML page containing GraphQL Playground.
/// This does not handle routing, so you can mount it on any endpoint.
pub fn playground_filter(
    graphql_endpoint_url: &'static str,
    subscriptions_endpoint_url: Option<&'static str>,
) -> warp::filters::BoxedFilter<(http::Response<Vec<u8>>,)> {
    warp::any()
        .map(move || playground_response(graphql_endpoint_url, subscriptions_endpoint_url))
        .boxed()
}

fn playground_response(
    graphql_endpoint_url: &'static str,
    subscriptions_endpoint_url: Option<&'static str>,
) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .header("content-type", "text/html;charset=utf-8")
        .body(
            juniper::http::playground::playground_source(
                graphql_endpoint_url,
                subscriptions_endpoint_url,
            )
            .into_bytes(),
        )
        .expect("response is valid")
}