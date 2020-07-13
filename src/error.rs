use juniper::{graphql_value, FieldError, IntoFieldError};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not identify environment {}", env))]
    #[snafu(visibility(pub))]
    Environment { env: String },

    #[snafu(display("Config Error: {}", msg))]
    #[snafu(visibility(pub))]
    ConfigError {
        msg: String,
        source: config::ConfigError,
    },

    #[snafu(display("lack of imagination: {}", msg))]
    #[snafu(visibility(pub))]
    MiscError { msg: String },

    #[snafu(display("Tokio IO Error: {}", msg))]
    #[snafu(visibility(pub))]
    TokioIOError {
        msg: String,
        source: tokio::io::Error,
    },

    #[snafu(display("Std IO Error: {}", msg))]
    #[snafu(visibility(pub))]
    IOError { msg: String, source: std::io::Error },

    #[snafu(display("JSON Error: {} - {}", msg, source))]
    #[snafu(visibility(pub))]
    JSONError {
        msg: String,
        source: serde_json::Error,
    },
}

impl IntoFieldError for Error {
    fn into_field_error(self) -> FieldError {
        match self {
            err @ Error::Environment { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Environment Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::ConfigError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new("Config Error", graphql_value!({ "internal_error": errmsg }))
            }

            err @ Error::MiscError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Miscellaneous Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::TokioIOError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Tokio IO Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::IOError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new("IO Error", graphql_value!({ "internal_error": errmsg }))
            }

            err @ Error::JSONError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new("JSON Error", graphql_value!({ "internal_error": errmsg }))
            }
        }
    }
}
