use std::{
    io::{self, BufRead},
    ops::RangeBounds,
};

use rand::Rng;

use crate::{
    board::{Board, Position},
    chess,
    movegen::MoveGenerator,
};

pub struct UCI {
    pub position: Position,
    pub running: bool,
}

#[derive(Debug)]
pub struct GoOptions {
    pub depth: Option<u8>,
    pub movetime: Option<u32>,
    pub infinite: bool,
}

impl UCI {
    pub fn new() -> UCI {
        let mut uci = UCI {
            position: Position::new(None),
            running: true,
        };
        uci.position.set_fen(chess::constants::STARTING_FEN);
        return uci;
    }

    pub fn uci_loop(&mut self) {
        while self.running {
            let stdin = io::stdin();
            let mut line = String::new();

            stdin.lock().read_line(&mut line).unwrap();

            let tokens = Iterator::collect::<Vec<&str>>(line.split_whitespace());

            if tokens.len() == 0 {
                continue;
            };

            match tokens[0] {
                "uci" => {
                    println!("id name redtail_vx");
                    println!("id author George T.G. Munyoro");
                    println!("uciok");
                }
                "go" => {
                    // store go command options
                    let mut options = GoOptions {
                        depth: None,
                        movetime: None,
                        infinite: false,
                    };

                    for i in 1..tokens.len() {
                        match tokens[i] {
                            "depth" => {
                                options.depth = Some(tokens[i + 1].parse::<u8>().unwrap());
                            }
                            "movetime" => {
                                options.movetime = Some(tokens[i + 1].parse::<u32>().unwrap());
                            }
                            "infinite" => {
                                options.infinite = true;
                            }
                            _ => {}
                        }
                    }
                }
                "stop" => {
                    let moves = self.position.generate_legal_moves();
                    let num = rand::thread_rng().gen_range(0..moves.len());
                    println!("bestmove {}", moves[num]);
                }
                "position" => self.handle_position_command(tokens),
                "quit" => self.stop(),
                "draw" => self.position.draw(),
                "ucinewgame" => self.position.set_fen(chess::constants::STARTING_FEN),
                "isready" => println!("readyok"),
                "listmoves" => {
                    let moves = self.position.generate_legal_moves();
                    for m in moves {
                        println!("{}", m);
                    }
                }
                _ => {
                    println!("Unknown command: {}", tokens[0]);
                }
            }
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    fn parse_move(&self, move_string: &str) -> Option<chess::Move> {
        let moves = self.position.generate_moves();

        for m in moves {
            if m.to_string() == move_string {
                return Some(m);
            }
        }

        return None;
    }

    fn parse_and_make_moves(&mut self, moves: Vec<&str>) {
        for m in moves {
            let m = self.parse_move(m);
            match m {
                Some(m) => {
                    self.position.make_move(m, false);
                }
                None => {}
            }
        }
    }

    fn handle_position_command(&mut self, tokens: Vec<&str>) {
        if tokens.len() < 2 {
            return;
        }

        if tokens[1] == "startpos" {
            self.position.set_fen(chess::constants::STARTING_FEN);

            if tokens.len() > 2 && tokens[2] == "moves" {
                self.parse_and_make_moves(tokens[3..].to_vec());
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

            self.position.set_fen(fen.as_str());

            if tokens.len() > 8 && tokens[8] == "moves" {
                self.parse_and_make_moves(tokens[9..].to_vec());
            }
        }
    }
}
