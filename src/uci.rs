use std::{
    io::{self, BufRead},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
};

use crate::{
    board::{Board, Position},
    chess::constants::STARTING_FEN,
    movegen::MoveGenerator,
    search::evaluate::*,
    search::options::*,
    search::utils::*,
    tt::TranspositionTable,
};

#[derive(Debug)]
enum UciPositionCommandKind {
    StartPos,
    Fen(String),
}

#[derive(Debug)]
enum UciCommand {
    Uci,
    IsReady,
    UciNewGame,
    Position(UciPositionCommandKind, Option<Vec<String>>),
    Go(SearchOptions),
    Stop,
    Quit,

    // Custom commands
    Perft(u8),
    Draw,
}

impl FromStr for UciCommand {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = s.split_whitespace().collect::<Vec<&str>>();

        let uci_command_kind = *tokens.get(0).ok_or_else(|| "Empty command")?;

        match uci_command_kind {
            "uci" => Ok(UciCommand::Uci),
            "isready" => Ok(UciCommand::IsReady),
            "ucinewgame" => Ok(UciCommand::UciNewGame),
            "stop" => Ok(UciCommand::Stop),
            "quit" => Ok(UciCommand::Quit),
            "go" => Ok(UciCommand::Go(SearchOptions::from(tokens.clone()))),

            "position" => {
                let position_command_kind_token =
                    *tokens.get(1).ok_or_else(|| "Invalid position command")?;

                let position_command_kind = match position_command_kind_token {
                    "startpos" => UciPositionCommandKind::StartPos,
                    "fen" => {
                        let mut fen = String::new();
                        for i in 2..8 {
                            fen.push_str(tokens[i]);
                            fen.push_str(" ");
                        }
                        fen.pop();
                        UciPositionCommandKind::Fen(fen)
                    }
                    _ => return Err("Invalid position command"),
                };

                let optional_moves_token_position = match position_command_kind {
                    UciPositionCommandKind::Fen(_) => 8,
                    UciPositionCommandKind::StartPos => 2,
                };

                let moves = if tokens
                    .get(optional_moves_token_position)
                    .is_some_and(|&t| t == "moves")
                {
                    Some(
                        tokens[optional_moves_token_position..]
                            .to_vec()
                            .iter()
                            .map(|t| t.to_string())
                            .collect(),
                    )
                } else {
                    None
                };

                Ok(UciCommand::Position(position_command_kind, moves))
            }

            // Custom commands
            "perft" => Ok(UciCommand::Perft(
                tokens
                    .get(1)
                    .ok_or_else(|| "Invalid perft command")?
                    .parse::<u8>()
                    .map_err(|_| "Invalid perft command")?,
            )),

            "draw" => Ok(UciCommand::Draw),

            _ => Err("Unknown command"),
        }
    }
}

pub struct UCI {
    is_searching: Arc<AtomicBool>,
    controller: mpsc::Sender<UciCommand>,
}

impl UCI {
    pub fn new() -> UCI {
        let (controller, rx) = mpsc::channel::<UciCommand>();
        let is_searching = Arc::new(AtomicBool::new(false));

        let thread_is_searching = is_searching.clone();
        std::thread::spawn(move || UCIController::run(rx, thread_is_searching));

        UCI {
            is_searching,
            controller,
        }
    }

    pub fn uci_loop(&mut self) {
        let stream = io::stdin().lock();
        for line in stream.lines() {
            match line {
                Ok(line) => match UciCommand::from_str(line.as_str()) {
                    Ok(UciCommand::Uci) => {
                        println!("id name redtail_vx");
                        println!("id author George T.G. Munyoro");
                        println!("uciok");
                    }

                    Ok(UciCommand::IsReady) => println!("readyok"),

                    Ok(UciCommand::Stop) => self.is_searching.store(false, Ordering::SeqCst),

                    Ok(UciCommand::Quit) => break,

                    Ok(command) => self.controller.send(command).unwrap(),

                    Err(e) => {
                        println!("Error: {}", e);
                    }
                },

                Err(e) => println!("Error: {}", e),
            }
        }
    }
}

struct UCIController();

impl UCIController {
    fn run(rx: mpsc::Receiver<UciCommand>, is_searching: Arc<AtomicBool>) {
        let mut position = Position::new(Some(STARTING_FEN));
        let shared_transposition_table = Box::new(TranspositionTable::new(2048));
        let mut evaluator = Evaluator::new(shared_transposition_table, is_searching.clone());

        for command in &rx {
            match command {
                UciCommand::Go(search_options) => {
                    is_searching.store(true, Ordering::SeqCst);
                    evaluator.get_best_move(&mut position, search_options, 0, 1);
                }

                UciCommand::Position(kind, moves) => {
                    position.set_fen(
                        match kind {
                            UciPositionCommandKind::StartPos => String::from(STARTING_FEN),
                            UciPositionCommandKind::Fen(fen) => String::from(fen),
                        }
                        .as_str(),
                    );

                    if let Some(moves) = moves {
                        for m in moves {
                            // TODO: Handle invalid moves gracefully
                            let bitpacked_move = parse_move(&mut position, m.as_str()).unwrap();
                            position.make_move(bitpacked_move, false);
                        }
                    }
                }

                UciCommand::UciNewGame => position.set_fen(STARTING_FEN),

                UciCommand::Draw => position.draw(),
                UciCommand::Perft(depth) => {
                    println!("Perft({}) = {}", depth, position.perft(depth))
                }

                _ => println!("Unknown command"),
            }
        }
    }
}
