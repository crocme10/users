use clap::{App, Arg};
use slog::{info, o, Drain, Logger};
use snafu::ResultExt;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use warp::{self, http, Filter};

use users::api::gql;
use users::api::user::User;
use users::error;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let matches = App::new("Microservice for users")
        .version("0.1")
        .author("Matthieu Paindavoine")
        .arg(
            Arg::with_name("address")
                .value_name("HOST")
                .short("h")
                .long("host")
                .default_value("localhost")
                .help("Address serving this server"),
        )
        .arg(
            Arg::with_name("port")
                .value_name("PORT")
                .short("p")
                .long("port")
                .default_value("8080")
                .help("Port"),
        )
        .get_matches();

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());

    let addr = matches
        .value_of("address")
        .ok_or_else(|| error::Error::MiscError {
            msg: String::from("Could not get address"),
        })?;

    let port = matches
        .value_of("port")
        .ok_or_else(|| error::Error::MiscError {
            msg: String::from("Could not get port"),
        })?;

    let port = port.parse::<u16>().map_err(|err| error::Error::MiscError {
        msg: format!("Could not parse into a valid port number ({})", err),
    })?;

    let users = tokio::fs::read_to_string("users.json")
        .await
        .context(error::TokioIOError {
            msg: String::from("Could not open users.json"),
        })?;
    let users: Vec<User> = serde_json::from_str(&users).context(error::JSONError {
        msg: String::from("Could not deserialize users.json content"),
    })?;
    let users: HashMap<String, String> = users.into_iter().map(|u| (u.username, u.email)).collect();

    run_server((addr, port), logger, users).await?;

    Ok(())
}

async fn run_server(
    addr: impl ToSocketAddrs,
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
