use std::{fs::File, io};

use ratatui::{backend::CrosstermBackend, Terminal};
use dotenv::dotenv;
use color_eyre::{eyre::Context, Result};
use tracing::info;

use crate::{
    app::{App, AppResult},
    event::{Event, EventHandler},
    tui::Tui,
};

use tracing_appender::{non_blocking, non_blocking::WorkerGuard};

pub mod app;
pub mod event;
pub mod tui;
pub mod ui;
pub mod widgets;
pub mod providers;
pub mod types;

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();
    let _guard = init_tracing()?;
    info!("started");
    // Create an application.

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);

    let mut app = App::new((&events).get_sender()).await;
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => app.handle_key_events(key_event).await,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::DataReceived(data) => app.handle_received_data(data),
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}

fn init_tracing() -> Result<WorkerGuard> {
    let file = File::create("tracing.log").wrap_err("failed to create tracing.log")?;
    let (non_blocking, guard) = non_blocking(file);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();
    Ok(guard)
}