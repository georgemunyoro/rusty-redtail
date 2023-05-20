use std::{
    io,
    sync::{Arc, Mutex},
};

use crate::{
    board::{self, Board, Position},
    chess,
    movegen::MoveGenerator,
    search::evaluate::*,
    tt::{self},
};

struct UCIOptions {
    debug: bool,
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
        let mut position = board::Position::new(None);
        position.set_fen(String::from(chess::constants::STARTING_FEN));
        let mut search_threads: Vec<std::thread::JoinHandle<()>> = vec![];
        let shared_transposition_table = Arc::new(Mutex::new(tt::TranspositionTable::new(2048)));
        let is_searching = Arc::new(Mutex::new(false));
        let mut uci_options = UCIOptions { debug: false };

        loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            let tokens = Iterator::collect::<Vec<&str>>(buffer.trim().split_whitespace());

            if tokens.len() == 0 {
                continue;
            };

            let shared_transposition_table = Arc::clone(&shared_transposition_table);
            let is_searching = Arc::clone(&is_searching);

            match tokens[0] {
                /*
                   UCI commands
                */
                // Print the engine info. No need to switch to uci mode
                // as the engine is always in uci mode.
                "uci" => {
                    println!("id name redtail_vx");
                    println!("id author George T.G. Munyoro");
                    println!("uciok");
                }

                // Check if the engine is ready
                "isready" => println!("readyok"),

                // Set debug mode
                "debug" => uci_options.debug = true,

                // Set the position to the starting position
                "ucinewgame" => self
                    .position
                    .set_fen(String::from(chess::constants::STARTING_FEN)),

                // Set the position
                "position" => {
                    if tokens.len() < 2 {
                        return;
                    }

                    if tokens[1] == "startpos" {
                        position.set_fen(String::from(chess::constants::STARTING_FEN));

                        // Handle moves
                        if tokens.len() > 2 && tokens[2] == "moves" {
                            parse_and_make_moves(&mut position, tokens[3..].to_vec());
                        }
                    }

                    if tokens[1] == "fen" && tokens.len() >= 8 {
                        let mut fen = String::new();
                        for i in 2..8 {
                            fen.push_str(tokens[i]);
                            fen.push_str(" ");
                        }
                        fen.pop();
                        position.set_fen(String::from(fen));

                        // Handle moves
                        if tokens.len() > 8 && tokens[8] == "moves" {
                            parse_and_make_moves(&mut position, tokens[9..].to_vec());
                        }
                    }
                }

                // Start the search
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

                    let tt = Arc::clone(&shared_transposition_table);
                    search_threads.push(self.create_search_thread(
                        self.position.as_fen(),
                        options,
                        is_searching,
                        tt,
                        0,
                    ));
                }

                // Stop the search and print the best move
                "stop" => {
                    {
                        let mut r = is_searching.lock().unwrap();
                        *r = false;
                    }

                    while search_threads.len() > 0 {
                        let cur_thread = search_threads.pop();
                        cur_thread.unwrap().join().unwrap();
                    }
                }

                // Quit the engine
                "quit" => {
                    break;
                }

                /*
                   Custom commands
                */
                // Print out perft information for given depth at the current position
                "perft" => {
                    if tokens.len() < 2 {
                        return;
                    }
                    let start_time = std::time::Instant::now();
                    let depth = tokens[1].parse::<u8>().unwrap();
                    let nodes = position.perft(depth);
                    let end_time = std::time::Instant::now();
                    let elapsed = end_time.duration_since(start_time);
                    let nps = nodes as f64 / (elapsed.as_millis() as f64 / 1000.0);
                    println!("nodes {} nps {}", nodes, nps);
                }

                // Print out an ASCII drawing of the board
                "draw" => position.draw(),

                _ => {
                    println!("Unknown command: {}", buffer.trim());
                }
            }
        }
    }

    fn create_search_thread(
        &self,
        position_fen: String,
        search_options: SearchOptions,
        is_searching: Arc<Mutex<bool>>,
        transposition_table: Arc<Mutex<tt::TranspositionTable>>,
        thread_id: usize,
    ) -> std::thread::JoinHandle<()> {
        let position_fen_clone = String::from(position_fen.clone());
        let is_searching = Arc::clone(&is_searching);
        let transpos_table_clone = Arc::clone(&transposition_table);

        return std::thread::spawn(move || {
            let mut eval_position = board::Position::new(None);
            eval_position.set_fen(String::from(position_fen_clone));

            let mut evaluator = Evaluator::new();
            evaluator.get_best_move(
                &mut eval_position,
                search_options,
                is_searching,
                transpos_table_clone,
                thread_id,
                1 + thread_id as u8,
            );
        });
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
