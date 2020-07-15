use clap::ArgMatches;
use slog::{info, Logger};

use users::error;
use users::settings::Settings;

pub async fn test<'a>(matches: &ArgMatches<'a>, logger: Logger) -> Result<(), error::Error> {
    info!(logger, "Testing application");
    let settings = Settings::new(matches)?;

    info!(logger, "Mode: {}", settings.mode);

    if settings.debug {
        info!(logger, "Debug: {}", settings.debug);
        info!(logger, "Database URL: {}", settings.database.url);
    }

    Ok(())
}
