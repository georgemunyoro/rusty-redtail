use std::{
    fmt::{Display, Formatter},
    io,
    sync::mpsc::{Receiver, TryRecvError},
};

use board::Board;

mod board;
mod chess;
mod evaluation;
mod movegen;
mod uci;
mod utils;

struct SearchThreadMessage {
    command: SearchThreadCommand,
    position: Option<board::Position>,
    options: Option<evaluation::SearchOptions>,
}

impl SearchThreadMessage {
    fn new(command: SearchThreadCommand) -> SearchThreadMessage {
        SearchThreadMessage {
            command,
            position: None,
            options: None,
        }
    }
}

enum SearchThreadCommand {
    Quit,
    Search,
    Stop,
    SetOption,
    SetPosition,
    Go,
    Unknown,
}

impl Display for SearchThreadCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchThreadCommand::Quit => write!(f, "Quit"),
            SearchThreadCommand::Search => write!(f, "Search"),
            SearchThreadCommand::Stop => write!(f, "Stop"),
            SearchThreadCommand::SetOption => write!(f, "SetOption"),
            SearchThreadCommand::SetPosition => write!(f, "SetPosition"),
            SearchThreadCommand::Go => write!(f, "Go"),
            SearchThreadCommand::Unknown => write!(f, "Unknown"),
        }
    }
}

fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<SearchThreadMessage>();

    let search_thread = std::thread::spawn(move || loop {
        let mut evaluator = evaluation::Evaluator::new();

        match rx.try_recv() {
            Ok(s) => {
                println!("Received: {}", s.command);

                match s.command {
                    SearchThreadCommand::Quit => {
                        break;
                    }
                    _ => {}
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                println!("Disconnected");
                break;
            }
        }
        sleep(100);
    });

    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        match buffer.trim() {
            "quit" => {
                println!("Quitting");
                tx.send(SearchThreadMessage::new(SearchThreadCommand::Quit))
                    .unwrap();
                break;
            }
            _ => {
                println!("Unknown command: {}", buffer.trim());
            }
        }
    }

    search_thread.join().unwrap();
}

fn sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
