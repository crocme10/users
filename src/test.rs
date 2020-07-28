use clap::ArgMatches;
use cucumber::{
    after, before, steps, CucumberBuilder, DefaultOutput, OutputVisitor, Scenario, Steps,
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use slog::{info, Logger};
use slog::{o, Drain};
use std::env;
use std::path::Path;
use std::thread;

use super::server::run_server;
use users::api::client::blocking::list_users;
use users::api::users::{MultiUsersResponseBody, SingleUserResponseBody};
use users::db::pg;
use users::error;
use users::settings::Settings;
use users::utils::{construct_headers, get_database_url, get_service_url};

pub async fn test<'a>(matches: &ArgMatches<'a>, logger: Logger) -> Result<(), error::Error> {
    let settings = Settings::new(matches)?;

    // FIXME There is work that should be done here to terminate the service
    // when we are done with testing.
    if settings.testing {
        info!(logger, "Launching testing service");
        let handle = tokio::runtime::Handle::current();
        let th = thread::spawn(move || {
            handle.spawn(async {
                let decorator = slog_term::TermDecorator::new().build();
                let drain = slog_term::FullFormat::new(decorator).build().fuse();
                let drain = slog_async::Async::new(drain).build().fuse();
                let logger = slog::Logger::root(drain, o!());
                let db_url =
                    env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL should be set");
                let pool = pg::connect(&db_url)
                    .await
                    .expect("Cannot obtain database connection for testing");
                info!(logger, "Running test service");
                run_server(settings, logger, pool).await;
            });
        });
        //th.join().expect("Waiting for test execution");
    }

    test_users();
    Ok(())
}

pub fn test_users() {
    let instance = get_cucumber_instance();

    let res = instance.run();

    if !res {
        std::process::exit(1);
    }
}

fn get_cucumber_instance() -> CucumberBuilder<MyWorld, DefaultOutput> {
    let output = DefaultOutput::new();
    let mut instance = CucumberBuilder::new(output);

    instance
        .features(vec![Path::new("./features/users").to_path_buf()])
        .steps(Steps::combine(vec![steps].iter().map(|f| f())));

    instance.setup(setup);

    let before_fns: Option<&[fn(&Scenario) -> ()]> = Some(&[a_before_fn]);
    if let Some(before) = before_fns {
        instance.before(before.to_vec());
    }

    let after_fns: Option<&[fn(&Scenario) -> ()]> = Some(&[an_after_fn]);
    if let Some(after) = after_fns {
        instance.after(after.to_vec());
    }

    instance
}

pub struct MyWorld {
    multi_resp: Option<MultiUsersResponseBody>,
    single_resp: Option<SingleUserResponseBody>,
    error: Option<String>,
}

impl cucumber::World for MyWorld {}

impl std::default::Default for MyWorld {
    fn default() -> MyWorld {
        // This function is called every time a new scenario is started
        MyWorld {
            multi_resp: None,
            single_resp: None,
            error: None,
        }
    }
}

steps!(MyWorld => {
    given "I have seeded the user database" |world, _step| {
    };

    when "I list users" |world, _step| {
        match list_users() {
            Ok(resp) => { world.multi_resp = Some(resp); }
            Err(err) => {
                println!("Could not deserialize server's response {}", err);
            }
        }
    };

    when "I add alice" |world, _step| {
        let data = "{ \
          \"query\": \"mutation addUser($user: UserRequestBody!) { \
              addUser(user: $user) { user { id, username, email, active, createdAt, updatedAt } } \
          }\", \
          \"variables\": { \
              \"user\": { \
                  \"username\": \"alice\", \
                  \"email\": \"alice@secret.org\" \
              } \
          } \
        }";
        let client = reqwest::blocking::Client::new();
        let url = get_service_url();
        match client.post(&url)
            .headers(construct_headers())
            .body(data)
            .send() {
                Ok(res) => {
                    let json: serde_json::Value = res.json().unwrap();
                    let res = &json["data"]["addUser"];
                    let value = res.clone();
                    let resp: Result<SingleUserResponseBody, _> = serde_json::from_value(value);
                    match resp {
                        Ok(resp) => { world.single_resp = Some(resp); }
                        Err(_err) => {
                            world.error = Some(format!("{}", json));
                        }
                    }
                }
                Err(err) => {
                    println!("Could not request users: {}", err);
                }
            }
    };

    then "I have no user in the response" |world, _step| {
        let resp = world.multi_resp.as_ref().unwrap();
        assert_eq!(resp.users_count,0);
    };

    then "I can verify the alice's details in the response" |world, _step| {
        let resp = world.single_resp.as_ref().unwrap();
        let user = resp.user.as_ref().unwrap();
        assert_eq!(user.username, "alice");
    };

    then "I get a duplicate username error" |world, _step| {
        let err = world.error.as_ref().unwrap();
        assert_ne!(err.find("Operation violates uniqueness constraint: Key (username)"), None);
    };

});

// Declares a before handler function named `a_before_fn`
before!(a_before_fn => |_scenario| {

});

// Declares an after handler function named `an_after_fn`
after!(an_after_fn => |_scenario| {

});

// A setup function to be called before everything else
pub fn setup() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    info!(logger, "Test Setup");
    // FIXME
    let db_url = get_database_url();
    let handle = tokio::runtime::Handle::current();
    let th = std::thread::spawn(move || {
        handle.block_on(async {
            pg::init_db(&db_url, logger)
                .await
                .expect("Could not initialize test database");
        })
    });
    th.join()
        .expect("Waiting for DB Initialization to complete");
}
