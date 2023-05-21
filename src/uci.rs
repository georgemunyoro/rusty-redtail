use std::{
    io,
    sync::{Arc, Mutex},
};

use crate::{
    board::{self, Board, Position},
    chess,
    movegen::MoveGenerator,
    search::evaluate::*,
    search::options::*,
    search::utils::*,
    tt,
};

pub struct UCI {
    position: Position,
    shared_transposition_table: Arc<Mutex<tt::TranspositionTable>>,
    is_searching: Arc<Mutex<bool>>,
    search_threads: Vec<std::thread::JoinHandle<()>>,
}

impl UCI {
    pub fn new() -> UCI {
        let mut uci = UCI {
            position: Position::new(None),
            shared_transposition_table: Arc::new(Mutex::new(tt::TranspositionTable::new(2048))),
            is_searching: Arc::new(Mutex::new(false)),
            search_threads: vec![],
        };
        uci.position
            .set_fen(String::from(chess::constants::STARTING_FEN));
        return uci;
    }

    pub fn uci_loop(&mut self) {
        loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            let tokens = Iterator::collect::<Vec<&str>>(buffer.trim().split_whitespace());

            if tokens.len() == 0 {
                continue;
            };

            /*
            * For reference to the UCI protocol, see:
            * https://www.wbec-ridderkerk.nl/html/UCIProtocol.html
            */
            match tokens[0] {
                "uci" => {
                    println!("id name redtail_vx");
                    println!("id author George T.G. Munyoro");
                    println!("uciok");
                }

                "isready" => println!("readyok"),

                "ucinewgame" => self
                    .position
                    .set_fen(String::from(chess::constants::STARTING_FEN)),

                "position" => self.handle_position(tokens),

                "go" => self.go(tokens),

                "stop" => self.stop_searching(),

                "quit" => break,

                // The rest of the commands below are custom convenience
                // commands. Mostly used for debugging, but are useful beyond that.

                "perft" => self.perft(tokens),

                "draw" => self.position.draw(),

                _ => {
                    println!("Unknown command: {}", buffer.trim());
                }
            }
        }
    }

    // Set the position
    fn handle_position(&mut self, tokens: Vec<&str>) {
        if tokens.len() < 2 {
            return;
        }

        if tokens[1] == "startpos" {
            self.position
                .set_fen(String::from(chess::constants::STARTING_FEN));

            // Handle moves
            if tokens.len() > 2 && tokens[2] == "moves" {
                parse_and_make_moves(&mut self.position, tokens[3..].to_vec());
            }
        }

        if tokens[1] == "fen" && tokens.len() >= 8 {
            let mut fen = String::new();
            for i in 2..8 {
                fen.push_str(tokens[i]);
                fen.push_str(" ");
            }
            fen.pop();
            self.position.set_fen(String::from(fen));

            // Handle moves
            if tokens.len() > 8 && tokens[8] == "moves" {
                parse_and_make_moves(&mut self.position, tokens[9..].to_vec());
            }
        }
    }

    /// Stops any current searches and prints the best move if any
    fn stop_searching(&mut self) {
        {
            let mut r = self.is_searching.lock().unwrap();
            *r = false;
        }

        while self.search_threads.len() > 0 {
            let cur_thread = self.search_threads.pop();
            cur_thread.unwrap().join().unwrap();
        }
    }

    /// Prints perft stats for the current position at the given depth
    fn perft(&mut self, tokens: Vec<&str>) {
        if tokens.len() < 2 {
            return;
        }
        let depth = tokens[1].parse::<u8>().unwrap();
        let start_time = std::time::Instant::now();
        let nodes = self.position.perft(depth);
        let end_time = std::time::Instant::now();
        let elapsed = end_time.duration_since(start_time);
        let nps = nodes as f64 / (elapsed.as_millis() as f64 / 1000.0);
        println!("nodes {} nps {}", nodes, nps);
    }

    /// Start searching with given options
    fn go(&mut self, tokens: Vec<&str>) {
        let options = SearchOptions::from(tokens);
        let tt = Arc::clone(&self.shared_transposition_table);
        let is_searching = Arc::clone(&self.is_searching);
        self.search_threads.push(self.create_search_thread(
            self.position.as_fen(),
            options,
            is_searching,
            tt,
            0,
        ));
    }

    /// Creates and starts a search thread
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
