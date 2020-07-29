use clap::ArgMatches;
use cucumber::{
    after, before, steps, CucumberBuilder, DefaultOutput, OutputVisitor, Scenario, Steps,
};
use futures::future::TryFutureExt;
use slog::{info, Logger};
use slog::{o, Drain};
use snafu::futures::try_future::TryFutureExt as SnafuTryFutureExt;
use std::env;
use std::path::Path;
use std::thread;

use super::server::run_server;
use users::api::client::blocking::{add_user, find_user_by_username, list_users};
use users::api::users::{MultiUsersResponseBody, SingleUserResponseBody, UserRequestBody};
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
        thread::spawn(move || {
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
    given "I have initialized the user database" |_world, _step| {
        // FIXME Not doing anything, that's a smell....
    };

    given regex r"I have a user with username (.*) and email (.*)$" |world, matches, _step| {
        let user = UserRequestBody {
            username: matches[1].clone(),
            email: matches[2].clone()
        };
        match add_user(user) {
            Ok(resp) => { world.single_resp = Some(resp); }
            Err(err) => { world.error = Some(format!("{}", err)); }
        }
    };

    when "I list users" |world, _step| {
        match list_users() {
            Ok(resp) => { world.multi_resp = Some(resp); }
            Err(err) => {
                println!("Could not deserialize server's response {}", err);
            }
        }
    };

    when regex r"I add a new user with username (.*) and email (.*)$" |world, matches, _step| {
        let user = UserRequestBody {
            username: matches[1].clone(),
            email: matches[2].clone()
        };
        match add_user(user) {
            Ok(resp) => { world.single_resp = Some(resp); }
            Err(err) => { world.error = Some(format!("{}", err)); }
        }
    };

    when regex r"I add a new user with no username and email (.*)$" |world, matches, _step| {
        let user = UserRequestBody {
            username: String::from(""),
            email: matches[1].clone()
        };
        match add_user(user) {
            Ok(resp) => { world.single_resp = Some(resp); }
            Err(err) => { world.error = Some(format!("{}", err)); }
        }
    };

    when r"I add a new user with an empty payload" |world, _step| {
        let handle = tokio::runtime::Handle::current();
        let th = std::thread::spawn(move || handle.block_on(async {
            let data = format!(
                r#"{{ "query": {query}, "variables": {{ "user": {variables} }} }}"#,
                query = "{}",
                variables = "{}"
            );
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
                async {
                    // We're expecting an error in the response
                    // FIXME: The error is actually a Warp rejection of the content-header.
                    // .... which I don't understand, since the `data` is valid JSON.
                    // I don't want to spend too much time on that right now, but it needs
                    // to be investigated.... Ideally I'd like some juniper error that says
                    // that the payload is invalid.
                    let txt = resp.text().await.unwrap();
                    let res: Result<(), _> = Err(error::Error::MiscError {
                        msg: txt
                    });
                    res
                }
            }).await
        } )
        );

        let res = th.join().unwrap();
        assert!(res.is_err());
        world.error = Some(format!("{}", res.unwrap_err()));
    };

    when regex r"I search for a user with username (.*)$" |world, matches, _step| {
        let username = matches[1].clone();
        match find_user_by_username(username) {
            Ok(resp) => { world.single_resp = Some(resp); }
            Err(err) => { world.error = Some(format!("{}", err)); }
        }
    };

    then regex r"the response's users count is (.*)$" |world, matches, _step| {
        let count = matches[1].parse::<i32>().unwrap();
        let resp = world.multi_resp.as_ref().unwrap();
        assert_eq!(resp.users_count,count);
    };

    then regex r"I can verify the username (.*) in the response" |world, matches, _step| {
        let username = matches[1].clone();
        let resp = world.single_resp.as_ref().unwrap();
        let user = resp.user.as_ref().unwrap();
        assert_eq!(user.username, username);
    };

    then "I get a duplicate username error" |world, _step| {
        let err = world.error.as_ref().unwrap();
        assert_ne!(err.find("Operation violates uniqueness constraint: Key (username)"), None);
    };

    then "I get a model violation error" |world, _step| {
        let err = world.error.as_ref().unwrap();
        assert_ne!(err.find("Operation violates model"), None);
    };

    then "I get an invalid request error" |world, _step| {
        let err = world.error.as_ref().unwrap();
        assert_ne!(err.find("Invalid request header"), None);
    };

    then "I can verify the user does not exists" |world, _step| {
        let resp = world.single_resp.as_ref().unwrap();
        assert!(resp.user.is_none())
    };

});

// Declares a before handler function named `a_before_fn`
before!(a_before_fn => |_scenario| {
    setup()
});

// Declares an after handler function named `an_after_fn`
after!(an_after_fn => |_scenario| {

});

// A setup function to be called before everything else
pub fn setup() {
    let logger = slog::Logger::root(slog::Discard, o!());
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
