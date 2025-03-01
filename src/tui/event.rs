use std::{sync::mpsc::{self, RecvError}, thread, time::Duration};
use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers, MouseEvent, KeyEventKind, KeyEventState};

#[derive(Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    sender: mpsc::Sender<Event>, // Keep sender in scope to prevent premature channel closure
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        let tick_rate = Duration::from_millis(tick_rate);

        let tick_sender = sender.clone();
        let event_sender = sender.clone();

        thread::spawn(move || {
            let mut last_tick = std::time::Instant::now();
            loop {
                if tick_sender.send(Event::Tick).is_err() {
                    break;
                }

                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(tick_rate);

                match event::poll(timeout) {
                    Ok(true) => {
                        match event::read() {
                            Ok(event::Event::Key(key)) => {
                                if event_sender.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            }
                            Ok(event::Event::Mouse(mouse)) => {
                                if event_sender.send(Event::Mouse(mouse)).is_err() {
                                    break;
                                }
                            }
                            Ok(event::Event::Resize(w, h)) => {
                                if event_sender.send(Event::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(false) => {
                    }
                    Err(_) => {
                        // Error polling events - likely interrupted
                        break;
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Self { receiver, sender }
    }

    pub fn next(&self) -> Result<Event> {
        match self.receiver.recv() {
            Ok(event) => Ok(event),
            Err(RecvError) => {
                Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }))
            }
        }
    }
}