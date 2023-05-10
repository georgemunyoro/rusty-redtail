use std::io::{self, BufRead};

use crate::{
    board::{Board, Position},
    chess,
    evaluation::Evaluator,
    movegen::MoveGenerator,
};

pub struct UCI {
    pub position: Position,
    pub evaluator: Evaluator,
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
            evaluator: Evaluator {
                running: false,
                transposition_table: std::collections::HashMap::new(),
                result: crate::evaluation::PositionEvaluation {
                    score: 0,
                    best_move: None,
                    depth: 0,
                    ply: 0,
                    nodes: 0,
                },
                killer_moves: [[chess::NULL_MOVE; 64]; 2],
                history_moves: [[0; 64]; 12],
            },
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

                    let depth = match options.depth {
                        Some(depth) => depth as u32,
                        None => 5,
                    };

                    let best_move = self.evaluator.get_best_move(&mut self.position, depth);

                    match best_move {
                        Some(best_move) => {
                            println!(
                                "info score {} depth {} nodes {}",
                                self.evaluator.result.score,
                                self.evaluator.result.depth,
                                self.evaluator.result.nodes
                            );
                            println!("bestmove {}", best_move);
                        }
                        None => {}
                    }
                }
                "stop" => {
                    let best_move = self.evaluator.get_best_move(&mut self.position, 5);
                    println!("{:?}", self.evaluator.result);
                    println!("bestmove {}", best_move.unwrap());
                }
                "perft" => {
                    let depth = tokens[1].parse::<u8>().unwrap();
                    let nodes = self.position.perft(depth);

                    println!("Nodes: {}", nodes);
                }
                "position" => self.handle_position_command(tokens),
                "quit" => self.stop(),
                "draw" => self.position.draw(),
                "ucinewgame" => self.position.set_fen(chess::constants::STARTING_FEN),
                "isready" => println!("readyok"),
                "hash" => println!("0x{:016x}u64", self.position.hash),
                "listmoves" => {
                    let mut moves = self.position.generate_legal_moves();
                    println!();
                    self.evaluator.order_moves(&mut moves);
                    for m in moves {
                        println!("{} {}", m, self.evaluator.get_move_mvv_lva(m));
                    }
                }
                "evaluate" => {
                    self.evaluator.result = crate::evaluation::PositionEvaluation {
                        score: 0,
                        best_move: None,
                        depth: 0,
                        ply: 0,
                        nodes: 0,
                    };
                    let moves = self.position.generate_moves();
                    // println!("{}", self.evaluator.evaluate(&mut self.position));
                    for m in moves {
                        let is_valid = self.position.make_move(m, false);
                        if !is_valid {
                            continue;
                        }

                        let score =
                            -self
                                .evaluator
                                .negamax(&mut self.position, -1000000, 1000000, 4);
                        self.position.unmake_move();

                        println!("{} {}", m, score);
                    }
                    self.evaluator.result = crate::evaluation::PositionEvaluation {
                        score: 0,
                        best_move: None,
                        depth: 0,
                        ply: 0,
                        nodes: 0,
                    };
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
