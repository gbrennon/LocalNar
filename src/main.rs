use localnar_presentation::tui::{TuiLaunchError, TuiLauncher};

#[tokio::main]
async fn main() -> Result<(), TuiLaunchError> {
    TuiLauncher::launch().await
}
