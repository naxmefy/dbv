use color_eyre::Result;

mod app;
mod errors;
mod tui;

fn main() -> Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;
    let app_result = app::App::default().run(&mut terminal);
    tui::restore()?;
    app_result
}