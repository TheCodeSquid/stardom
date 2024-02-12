use std::process::ExitCode;

use stardom_cli::{is_error_silent, shell};

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = stardom_cli::run().await {
        if is_error_silent(&err) {
            // No additional printing
        } else if let Some(cargo_metadata::Error::CargoMetadata { stderr }) = err.downcast_ref() {
            shell().error(stderr);
        } else {
            shell().error(err);
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
