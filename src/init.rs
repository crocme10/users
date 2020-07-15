use clap::ArgMatches;
use slog::{debug, info, Logger};
use snafu::ResultExt;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use users::error;
use users::settings::Settings;

pub async fn init<'a>(matches: &ArgMatches<'a>, logger: Logger) -> Result<(), error::Error> {
    info!(logger, "Initiazing application");
    let settings = Settings::new(matches)?;

    info!(logger, "Mode: {}", settings.mode);

    if settings.debug {
        info!(logger, "Debug: {}", settings.debug);
        info!(logger, "Database URL: {}", settings.database.url);
    }

    info!(logger, "Initializing {}", settings.database.url);

    init_db(settings, logger).await
}

async fn init_db(settings: Settings, logger: Logger) -> Result<(), error::Error> {
    // This is essentially running 'psql $DATABASE_URL < db/init.sql', and logging the
    // psql output
    let mut cmd = Command::new("psql");
    cmd.arg(settings.database.url);
    cmd.stdout(Stdio::piped());
    let file = std::fs::File::open("db/init.sql").expect("file");
    cmd.stdin(Stdio::from(file));

    let mut child = cmd.spawn().context(error::TokioIOError {
        msg: format!("Failed to execute psql"),
    })?;

    let stdout = child.stdout.take().ok_or(error::Error::MiscError {
        msg: format!("child did not have a handle to stdout"),
    })?;

    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        // FIXME Need to do something about logging this and returning an error.
        let status = child.await.expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    while let Some(line) = reader.next_line().await.context(error::TokioIOError {
        msg: String::from("Could not read from piped output"),
    })? {
        debug!(logger, "psql {}", line);
    }

    Ok(())
}
