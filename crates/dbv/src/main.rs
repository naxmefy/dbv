use std::error::Error;

use clap::Parser;
use color_eyre::Result;
use tracing::debug;

use crate::cli::Args;

mod app;
mod cli;
mod errors;
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    debug!("Arg: {:?}", args);

    match &args.command {
        _ => run_app()
    }.expect("TODO: panic message");

    Ok(())
}

fn run_app() -> Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;
    let app_result = app::App::default().run(&mut terminal);
    tui::restore()?;
    app_result
}