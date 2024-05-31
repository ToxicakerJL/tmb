mod app;
mod component;
mod components;
mod core;
mod utils;

use std::io::stdout;
use std::panic;
use color_eyre::config::HookBuilder;
use color_eyre::eyre;
use color_eyre::eyre::Result;
use crossterm::terminal::LeaveAlternateScreen;
use crate::app::App;
use crate::components::game_page::GamePage;
use crate::components::home_page::HomePage;
use crate::components::select_boss_page::SelectBossPage;

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