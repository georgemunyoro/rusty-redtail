use std::{
    io,
    sync::{mpsc::TryRecvError, Arc, Mutex},
};

use crate::{
    board::{self, Board, Position},
    chess,
    movegen::MoveGenerator,
    search::evaluate::*,
    tt::{self},
};

struct SearchThreadMessage {
    command: SearchThreadCommand,

    /// We send the position as an FEN string, so we don't have to worry about
    /// cloning it and sending it across threads.
    position: Option<String>,
    options: Option<SearchOptions>,
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
    Stop,
    SetPosition,
    Go,
}

pub struct UCI {
    position: Position,
}

impl UCI {
    pub fn new() -> UCI {
        let mut uci = UCI {
            position: Position::new(None),
        };
        uci.position
            .set_fen(String::from(chess::constants::STARTING_FEN));
        return uci;
    }

    pub fn uci_loop(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel::<SearchThreadMessage>();

        let search_thread = std::thread::spawn(move || {
            let mut position = board::Position::new(None);
            position.set_fen(String::from(chess::constants::STARTING_FEN));

            let transposition_table = Arc::new(Mutex::new(tt::TranspositionTable::new(4096)));

            let running = Arc::new(Mutex::new(true));
            let mut eval_handles = vec![];

            loop {
                let is_evaluating = Arc::clone(&running);
                let transpos_table = Arc::clone(&transposition_table);

                match rx.try_recv() {
                    Ok(s) => match s.command {
                        SearchThreadCommand::Quit => {
                            break;
                        }
                        SearchThreadCommand::SetPosition => match s.position {
                            Some(fen) => position.set_fen(fen),
                            None => {}
                        },
                        SearchThreadCommand::Go => match s.options {
                            Some(options) => {
                                let position_fen = position.as_fen().to_string();

                                for thread_id in 0..4 {
                                    let position_fen_clone = String::from(position_fen.clone());
                                    let is_evaluating_clone = Arc::clone(&is_evaluating);
                                    let transpos_table_clone = Arc::clone(&transpos_table);

                                    let curr_thread_handle = std::thread::spawn(move || {
                                        let mut eval_position = board::Position::new(None);
                                        eval_position.set_fen(String::from(position_fen_clone));

                                        let mut evaluator = Evaluator::new();
                                        let bestmove = evaluator.get_best_move(
                                            &mut eval_position,
                                            options,
                                            is_evaluating_clone,
                                            transpos_table_clone,
                                            thread_id,
                                            thread_id as u8 + 1,
                                        );

                                        return bestmove;
                                    });

                                    eval_handles.push(curr_thread_handle);
                                }
                            }
                            None => {}
                        },
                        SearchThreadCommand::Stop => {
                            {
                                let mut r = running.lock().unwrap();
                                *r = false;
                            }

                            while eval_handles.len() > 0 {
                                let cur_thread = eval_handles.pop();
                                cur_thread.unwrap().join().unwrap();
                            }
                        }
                    },
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        println!("Disconnected");
                        break;
                    }
                }
            }
            sleep(100);
        });

        let mut position = board::Position::new(None);
        position.set_fen(String::from(chess::constants::STARTING_FEN));

        loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            let tokens = Iterator::collect::<Vec<&str>>(buffer.trim().split_whitespace());

            if tokens.len() == 0 {
                continue;
            };

            match tokens[0] {
                "perft" => {
                    if tokens.len() < 2 {
                        return;
                    }
                    let depth = tokens[1].parse::<u8>().unwrap();
                    let nodes = position.perft(depth);
                    println!("nodes: {}", nodes);
                }
                "uci" => {
                    println!("id name redtail_vx");
                    println!("id author George T.G. Munyoro");
                    println!("uciok");
                }
                "quit" => {
                    tx.send(SearchThreadMessage::new(SearchThreadCommand::Quit))
                        .unwrap();
                    break;
                }
                "ucinewgame" => {
                    self.position
                        .set_fen(String::from(chess::constants::STARTING_FEN));
                }
                "isready" => println!("readyok"),
                "position" => {
                    if tokens.len() < 2 {
                        return;
                    }

                    if tokens[1] == "startpos" {
                        position.set_fen(String::from(chess::constants::STARTING_FEN));

                        if tokens.len() > 2 && tokens[2] == "moves" {
                            parse_and_make_moves(&mut position, tokens[3..].to_vec());
                        }
                    }

                    if tokens[1] == "fen" {
                        if tokens.len() < 8 {
                            return;
                        }

                        let mut fen = String::new();
                        for i in 2..8 {
                            fen.push_str(tokens[i]);
                            fen.push_str(" ");
                        }
                        fen.pop();

                        position.set_fen(String::from(fen));

                        if tokens.len() > 8 && tokens[8] == "moves" {
                            parse_and_make_moves(&mut position, tokens[9..].to_vec());
                        }
                    }

                    tx.send(SearchThreadMessage {
                        command: SearchThreadCommand::SetPosition,
                        position: Some(position.as_fen()),
                        options: None,
                    })
                    .unwrap();
                }
                "go" => {
                    let mut options = SearchOptions::new();
                    for i in 1..tokens.len() {
                        match tokens[i] {
                            "infinite" => options.infinite = true,
                            "depth" => options.depth = Some(tokens[i + 1].parse::<u8>().unwrap()),
                            "binc" => options.binc = Some(tokens[i + 1].parse::<u32>().unwrap()),
                            "winc" => options.winc = Some(tokens[i + 1].parse::<u32>().unwrap()),
                            "btime" => options.btime = Some(tokens[i + 1].parse::<u32>().unwrap()),
                            "wtime" => options.wtime = Some(tokens[i + 1].parse::<u32>().unwrap()),
                            "movestogo" => {
                                options.movestogo = Some(tokens[i + 1].parse::<u32>().unwrap())
                            }
                            "movetime" => {
                                options.movetime = Some(tokens[i + 1].parse::<u32>().unwrap())
                            }
                            _ => {}
                        }
                    }

                    tx.send(SearchThreadMessage {
                        command: SearchThreadCommand::Go,
                        position: Some(position.as_fen()),
                        options: Some(options),
                    })
                    .unwrap();
                }
                "stop" => {
                    tx.send(SearchThreadMessage::new(SearchThreadCommand::Stop))
                        .unwrap();
                }
                _ => {
                    println!("Unknown command: {}", buffer.trim());
                }
            }
        }

        search_thread.join().unwrap();
    }
}

pub fn parse_and_make_moves(position: &mut board::Position, moves: Vec<&str>) {
    for m in moves {
        let m = parse_move(position, m);
        match m {
            Some(m) => {
                position.make_move(m, false);
            }
            None => {}
        }
    }
}

pub fn parse_move(
    position: &mut board::Position,
    move_string: &str,
) -> Option<chess::_move::BitPackedMove> {
    let moves = position.generate_moves();

    for m in moves {
        if m.to_string() == move_string {
            return Some(m);
        }
    }

    return None;
}

fn sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
