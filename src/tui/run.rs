use std::io;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use super::app::App;
use super::event::{Event, EventHandler};
use super::ui::UI;

pub fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let ui = UI::new();
    let events = EventHandler::new(250); // 250ms polling rate
    let res = run_app(&mut terminal, &mut app, &events, &ui);

    // Restore terminal
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
    ui: &UI,
) -> Result<()> {
    let size = terminal.size()?;
    app.update_dimensions(size.height);

    loop {
        terminal.draw(|frame| ui.render(app, frame))?;

        match events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                if app.handle_key(key_event)? {
                    return Ok(());
                }
            }
            Event::Mouse(mouse_event) => {
                if app.handle_mouse(mouse_event)? {
                    return Ok(());
                }
            }
            Event::Resize(width, height) => {
                terminal.resize(ratatui::prelude::Rect::new(0, 0, width, height))?;
                terminal.clear()?;
                app.update_dimensions(height);
            }
        }
    }
}