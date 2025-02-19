use std::{sync::mpsc, thread, time::Duration};
use anyhow::Result;
use crossterm::event::{self, KeyEvent, MouseEvent};

#[derive(Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        let tick_rate = Duration::from_millis(tick_rate);

        thread::spawn(move || {
            let mut last_tick = std::time::Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(tick_rate);

                if event::poll(timeout).expect("Failed to poll events") {
                    match event::read().expect("Failed to read event") {
                        event::Event::Key(key) => {
                            if let Err(err) = sender.send(Event::Key(key)) {
                                eprintln!("Failed to send key event: {}", err);
                                return;
                            }
                        }
                        event::Event::Mouse(mouse) => {
                            if let Err(err) = sender.send(Event::Mouse(mouse)) {
                                eprintln!("Failed to send mouse event: {}", err);
                                return;
                            }
                        }
                        event::Event::Resize(w, h) => {
                            if let Err(err) = sender.send(Event::Resize(w, h)) {
                                eprintln!("Failed to send resize event: {}", err);
                                return;
                            }
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Err(err) = sender.send(Event::Tick) {
                        eprintln!("Failed to send tick event: {}", err);
                        return;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Self { receiver }
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}