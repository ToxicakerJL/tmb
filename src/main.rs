mod app;
mod component;
mod components;
mod core;
mod utils;

use std::io::stdout;
use std::panic;
use std::path::PathBuf;
use color_eyre::config::HookBuilder;
use color_eyre::eyre;
use color_eyre::eyre::Result;
use crossterm::terminal::LeaveAlternateScreen;
use lazy_static::lazy_static;
use tracing::Level;
use tracing_error::ErrorLayer;
use crate::app::App;
use crate::components::game_page::GamePage;
use crate::components::home_page::HomePage;
use crate::components::select_boss_page::SelectBossPage;
use crate::utils::get_project_root_path;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

lazy_static! {
  pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
  pub static ref LOG_ENV: String = format!("{}_LOGLEVEL", PROJECT_NAME.clone());
  pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}

async fn tokio_main() -> Result<()> {
    initialize_logging()?;
    initialize_panic_handler()?;
    let mut app = App::new()?;
    app.register_component(components::home_page::NAME.to_string(), Box::new(HomePage::new()))?;
    app.register_component(components::select_boss_page::NAME.to_string(), Box::new(SelectBossPage::new()))?;
    app.register_component(components::game_page::NAME.to_string(), Box::new(GamePage::new()))?;
    app.run().await?;
    Ok(())
}

/// Rust groups errors into two major categories: recoverable and unrecoverable errors.
/// For a recoverable error, such as a file not found error, we most likely just want to report the problem to the user and retry the operation.
/// Unrecoverable errors are always symptoms of bugs, like trying to access a location beyond the end of an array, and so we want to immediately stop the program.
/// One approach that makes it easy to show unhandled errors is to use the color_eyre crate to augment the error reporting hooks.
/// Run `RUST_BACKTRACE=full cargo run` to see the full error stacktrace.
fn initialize_panic_handler() -> Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(stdout(), LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        panic_hook(panic_info);
    }));
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error: &(dyn std::error::Error + 'static)| {
        crossterm::execute!(stdout(), LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        eyre_hook(error)
    },
    ))?;
    Ok(())
}

/// Initialize logging framework for the project. The log file "tmp.log" will be generated for debug purpose
/// Usage:
/// ```
/// info!("aaa");
/// info!("ccccc");
/// debug!("hello");
/// ```
pub fn initialize_logging() -> Result<()> {
    let directory = PathBuf::from(get_project_root_path());
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join(LOG_FILE.clone());
    let log_file = std::fs::File::create(log_path)?;
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var(LOG_ENV.clone()))
            .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
    );
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::LevelFilter::from(Level::INFO));
    tracing_subscriber::registry().with(file_subscriber).with(ErrorLayer::default()).init();
    Ok(())
}