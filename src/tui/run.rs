use std::io;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::settings::UserSettings;

use super::app::App;
use super::event::{Event, EventHandler};
use super::ui::UI;

pub fn run_tui(settings: Option<UserSettings>) -> Result<()> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::with_settings(settings)?;
    let mut ui = UI::new();
    let events = EventHandler::new(250);
    let res = run_app(&mut terminal, &mut app, &events, &mut ui);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stderr>>,
    app: &mut App,
    events: &EventHandler,
    ui: &mut UI,
) -> Result<()> {
    let size = terminal.size()?;
    ui.update_dimensions(app, size.height);

    loop {
        terminal.draw(|frame| ui.render(app, frame))?;
        app.process_async_results();

        match events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                if app.handle_key(ui, key_event)? {
                    return Ok(());
                }
            }
            Event::Mouse(mouse_event) => {
                if app.handle_mouse(ui, mouse_event)? {
                    return Ok(());
                }
            }
            Event::Resize(width, height) => {
                terminal.resize(ratatui::prelude::Rect::new(0, 0, width, height))?;
                terminal.clear()?;
                ui.update_dimensions(app, height);
            }
        }
    }
}