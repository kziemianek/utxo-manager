use std::sync::mpsc;
use std::{io, thread};

use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use tui::backend::{Backend, CrosstermBackend};
use tui::widgets::{Block, Borders, List, ListItem, ListState};
use tui::{Frame, Terminal};

use crate::cli::read_cli;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use serde_json::json;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};

mod cli;

enum AppEvent<I> {
    Input(I),
    Tick,
}

#[derive(Serialize, Deserialize)]
struct RpcMethod {
    jsonrpc: String,
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct ListUnspentResponse {
    result: Vec<Unspent>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Unspent {
    txid: String,
    vout: u8,
}

struct Options {
    rpc_host: String,
    rpc_port: String,
    rpc_user: String,
    rpc_pass: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = read_cli();
    // let tx_to_omit = get_argument(&matches, "omit-tx");
    enable_raw_mode()?;
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

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let unspents = get_unspents(&options);
    let mut utxo_list_state = ListState::default();

    loop {
        let list_items: Vec<ListItem> = unspents
            .iter()
            .map(|u| ListItem::new(u.txid.to_owned()))
            .collect();
        terminal.draw(|f| ui(f, list_items, &mut utxo_list_state))?;

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
                        Option::Some(idx) => {
                            let mut next_idx;
                            if idx == unspents.len() {
                                next_idx = 0;
                            } else {
                                next_idx = idx + 1;
                            }
                            utxo_list_state.select(Some(next_idx))
                        }
                        Option::None => utxo_list_state.select(Some(0)),
                    }
                }
                KeyCode::Up => {
                    let selected = utxo_list_state.selected();
                    let total_items = unspents.len();
                    match selected {
                        Option::Some(idx) => {
                            let mut next_idx;
                            if idx == 0 {
                                next_idx = unspents.len();
                            } else {
                                next_idx = idx - 1;
                            }
                            utxo_list_state.select(Some(next_idx))
                        }
                        Option::None => utxo_list_state.select(Some(total_items - 1)),
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
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, items: Vec<ListItem>, list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let list = List::new(items)
        .block(
            Block::default()
                .title("Current unspents")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
    f.render_stateful_widget(list, chunks[0], list_state);
}

fn get_unspents(options: &Options) -> Vec<Unspent> {
    //
    // let list_unspent = RpcMethod {
    //     jsonrpc: "1.0".to_owned(),
    //     id: "lock-unspents".to_owned(),
    //     method: "listunspent".to_owned(),
    //     params: serde_json::Value::Array(vec![
    //         json!(1),
    //         json!(999999),
    //         serde_json::Value::Array(vec![]),
    //         serde_json::Value::Bool(true),
    //     ]),
    // };
    //
    // let req = serde_json::to_string(&list_unspent).unwrap();
    // let client = reqwest::blocking::Client::new();
    // let node_url = "http://".to_owned() + &options.rpc_host + ":" + &options.rpc_port + "/";
    //
    // let resp: serde_json::Value = client
    //     .post(&node_url)
    //     .basic_auth(options.rpc_user.to_owned(), Some(options.rpc_pass.to_owned()))
    //     .body(req)
    //     .send()
    //     .unwrap()
    //     .json()
    //     .unwrap();
    // let unspents: ListUnspentResponse = serde_json::from_value(resp).unwrap();
    // unspents.result

    vec![
        Unspent {
            txid: "123".to_owned(),
            vout: 0,
        },
        Unspent {
            txid: "456".to_owned(),
            vout: 1,
        },
        Unspent {
            txid: "789".to_owned(),
            vout: 0,
        },
    ]
}

fn lock_unspent(unspent: &Unspent, options: &Options) {
    let client = reqwest::blocking::Client::new();
    let node_url = "http://".to_owned() + &options.rpc_host + ":" + &options.rpc_port + "/";

    let mut json = serde_json::Value::default();
    json["txid"] = json!(unspent.txid);
    json["vout"] = json!(unspent.vout);

    let list_unspent = RpcMethod {
        jsonrpc: "1.0".to_owned(),
        id: "lock-unspents".to_owned(),
        method: "lockunspent".to_owned(),
        params: serde_json::Value::Array(vec![json!(false), serde_json::Value::Array(vec![json])]),
    };
    let req = serde_json::to_string(&list_unspent).unwrap();
    client
        .post(&node_url)
        .basic_auth(
            options.rpc_user.to_owned(),
            Some(options.rpc_pass.to_owned()),
        )
        .body(req)
        .send()
        .unwrap();
}
