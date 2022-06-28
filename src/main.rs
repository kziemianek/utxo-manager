use std::sync::mpsc;
use std::{io, thread};

use crossterm::execute;
use crossterm::terminal::disable_raw_mode;

use std::time::{Duration, Instant};

pub use tui::backend::Backend;
use tui::widgets::{ListItem, ListState};

use crate::cli::read_cli;
use crossterm::event;
use crossterm::event::DisableMouseCapture;

use crate::utxo::{get_unspents, lock_unspent};
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::terminal::LeaveAlternateScreen;

mod app;
mod cli;
mod ui;
mod utxo;

enum AppEvent<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = read_cli();
    // let tx_to_omit = get_argument(&matches, "omit-tx");
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let Event::Key(key) = event::read().expect("can read events") {
                    tx.send(AppEvent::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(AppEvent::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });
    let mut terminal = ui::setup(io::stdout())?;
    let unspents = get_unspents(&options);
    let mut utxo_list_state = ListState::default();

    loop {
        let list_items: Vec<ListItem> = unspents
            .iter()
            .map(|u| ListItem::new(u.txid.to_owned()))
            .collect();
        terminal.draw(|f| ui::ui(f, list_items, &mut utxo_list_state))?;

        match rx.recv()? {
            AppEvent::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Down => {
                    let selected = utxo_list_state.selected();
                    match selected {
                        Some(idx) => {
                            let mut next_idx;
                            if idx == unspents.len() {
                                next_idx = 0;
                            } else {
                                next_idx = idx + 1;
                            }
                            utxo_list_state.select(Some(next_idx))
                        }
                        None => utxo_list_state.select(Some(0)),
                    }
                }
                KeyCode::Up => {
                    let selected = utxo_list_state.selected();
                    let total_items = unspents.len();
                    match selected {
                        Some(idx) => {
                            let mut next_idx;
                            if idx == 0 {
                                next_idx = unspents.len();
                            } else {
                                next_idx = idx - 1;
                            }
                            utxo_list_state.select(Some(next_idx))
                        }
                        None => utxo_list_state.select(Some(total_items - 1)),
                    }
                }
                KeyCode::Char('l') => lock_unspent(
                    &unspents.get(utxo_list_state.selected().unwrap()).unwrap(),
                    &options,
                ),
                KeyCode::Char('r') => {
                    println!("refresh")
                }
                _ => {}
            },
            AppEvent::Tick => {}
        }
    }

    thread::sleep(Duration::from_millis(5000));

    // restore terminal
    ui::cleanup(terminal)?;
    Ok(())
}
