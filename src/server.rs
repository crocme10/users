use clap::ArgMatches;
use slog::{info, o, Logger};
use snafu::ResultExt;
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::net::ToSocketAddrs;
use users::api::gql;
use users::db::pg;
use users::error;
use users::settings::Settings;
// use warp::http::header::{
//     ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
//     ACCESS_CONTROL_MAX_AGE,
// };
use warp::{self, http, Filter};

#[allow(clippy::needless_lifetimes)]
pub async fn run<'a>(matches: &ArgMatches<'a>, logger: Logger) -> Result<(), error::Error> {
    let settings = Settings::new(matches)?;
    let s2 = settings.clone();

    let clogger = logger.new(
        o!("host" => s2.service.host, "port" => s2.service.port, "database" => s2.database.url),
    );

    let db_url = settings.database.url.clone();

    let pool = pg::connect(&db_url).await.context(error::DBError {
        msg: String::from("foo"),
    })?;

    run_server(settings, clogger, pool).await
}

pub async fn run_server(
    settings: Settings,
    logger: Logger,
    pool: PgPool,
) -> Result<(), error::Error> {
    let logger1 = logger.clone();
    let pool1 = pool.clone();
    let state = warp::any().map(move || gql::Context {
        logger: logger1.clone(),
        pool: pool1.clone(),
    });

    let playground = warp::get()
        .and(warp::path("playground"))
        .and(playground_filter("/graphql", Some("/subscriptions")));

    let graphql_filter = juniper_warp::make_graphql_filter(gql::schema(), state.boxed());

    // let cors = warp::reply::with::header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[http::Method::POST])
        .allow_headers(vec!["content-type"]);
    // let options = warp::options()
    //     //.and(warp::path("graphql"))
    //     .and(warp::header("access-control-request-headers"))
    //     .and(warp::header("access-control-request-method"))
    //     .and_then(preflight_request)
    //     .with(warp::log("warp cors"));

    let graphql = warp::post()
        .and(warp::path("graphql"))
        .and(graphql_filter)
        .with(cors);

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

// async fn preflight_request(
//     headers: String,
//     methods: String,
// ) -> Result<impl warp::Reply, Infallible> {
//     let reply = warp::reply::with_header(warp::reply(), ACCESS_CONTROL_ALLOW_ORIGIN, "*");
//     let reply = warp::reply::with_header(reply, ACCESS_CONTROL_ALLOW_HEADERS, headers);
//     let reply = warp::reply::with_header(reply, ACCESS_CONTROL_ALLOW_METHODS, methods);
//     let reply = warp::reply::with_header(reply, ACCESS_CONTROL_MAX_AGE, "86400");
//     Ok(reply)
// }
