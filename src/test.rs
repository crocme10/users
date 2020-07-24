use clap::ArgMatches;
use cucumber::{
    after, before, steps, CucumberBuilder, DefaultOutput, OutputVisitor, Scenario, Steps, World,
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use slog::{info, Logger};
use std::collections::HashMap;
use std::path::Path;

use users::api::model::User;
use users::api::users::MultiUsersResponseBody;
use users::error;
use users::settings::Settings;

pub async fn test<'a>(matches: &ArgMatches<'a>, logger: Logger) -> Result<(), error::Error> {
    let settings = Settings::new(matches)?;
    // When running tests.... we should be in testing mode!
    if !settings.testing {
        return Err(error::Error::MiscError {
            msg: String::from(
                "When running tests, you should be in testing mode (Probably set RUN_MODE=test)",
            ),
        });
    }

    if settings.debug {
        info!(logger, "Database URL: {}", settings.database.url);
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
    // You can use this struct for mutable context in scenarios.
    users: HashMap<String, String>,
    resp: MultiUsersResponseBody,
}

impl cucumber::World for MyWorld {}

impl std::default::Default for MyWorld {
    fn default() -> MyWorld {
        // This function is called every time a new scenario is started
        MyWorld {
            users: HashMap::new(),
            resp: MultiUsersResponseBody {
                users: Vec::new(),
                users_count: 0,
            },
        }
    }
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

steps!(MyWorld => {
    given "I have seeded the user database" |world, _step| {
        // Here we assume the service has been seeded with 'users.json'
        let users = std::fs::read_to_string("users.json").expect("No users.json");
        let users: Vec<User> = serde_json::from_str(&users).expect("Cannot deserialize users.json");
        world.users = users.into_iter().map(|u| (u.username, u.email)).collect();
    };

    when "I list users" |world, _step| {
        let data = "{ \"query\": \"{ users { users { username, email }, usersCount } }\" }";
        let client = reqwest::blocking::Client::new();
        // match client.post("http://172.18.0.3:8081/graphql")
        // FIXME: Hardcoded service target... Use settings instead
        match client.post("http://users:8081/graphql")
            .headers(construct_headers())
            .body(data)
            .send() {
                Ok(res) => {
                    let json: serde_json::Value = res.json().unwrap();
                    let res = &json["data"]["users"];
                    let res = res.clone();
                    let res: Result<MultiUsersResponseBody, _> = serde_json::from_value(res);
                    match res {
                        Ok(resp) => { world.resp = resp; }
                        Err(err) => {
                            println!("Could not deserialize server's response {}", err);
                        }
                    }
                }
                Err(err) => {
                    println!("Could not request users: {}", err);
                }
            }
    };

    when "I add a new user" |_world, _step| {
        let data = "{ \"query\": \"mutation addUser($username: String!, $email: String!) { addUser(username: $username, email: $email) { username, email } }\", \
          \"variables\": { \"username\": \"alice\", \"email\": \"alice@secret.org\" } }";
        let client = reqwest::blocking::Client::new();
        match client.post("http://172.18.0.3:8081/graphql")
        // FIXME: Hardcoded service target... Use settings instead
        // match client.post("http://users:8081/graphql")
            .headers(construct_headers())
            .body(data)
            .send() {
                Ok(res) => {
                    println!("res: {:?}", res);
                    // let json: serde_json::Value = res.json().unwrap();
                    let text = res.text().unwrap();
                    println!("res: {:?}", text);
                    // let res = &json["data"]["users"];
                    // let res = res.clone();
                    // let res: Result<MultiUsersResponseBody, _> = serde_json::from_value(res);
                    // match res {
                    //     Ok(resp) => { world.resp = resp; }
                    //     Err(err) => {
                    //         println!("Could not deserialize server's response {}", err);
                    //     }
                    // }
                }
                Err(err) => {
                    println!("Could not request users: {}", err);
                }
            }
    };

    then "I have as many users in the database as in the response" |world, _step| {
        // Check that the outcomes to be observed have occurred
        assert_eq!(world.resp.users_count, world.users.len() as i32);
    };


});

// Declares a before handler function named `a_before_fn`
before!(a_before_fn => |_scenario| {

});

// Declares an after handler function named `an_after_fn`
after!(an_after_fn => |_scenario| {

});

// A setup function to be called before everything else
pub fn setup() {}
