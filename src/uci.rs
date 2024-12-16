use std::{
    ffi::CString,
    io,
    sync::{Arc, RwLock},
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

#[derive(Debug)]
struct Options {
    threads: usize,
    hash: usize,
}

pub struct UCI {
    position: Position,
    ll_shared_transposition_table: Arc<RwLock<tt::TranspositionTable>>,
    is_searching: Arc<RwLock<bool>>,
    search_threads: Vec<std::thread::JoinHandle<()>>,
    options: Options,
}

impl UCI {
    pub fn new() -> UCI {
        unsafe {
            nnue_init(CString::new("nns/nn-04cf2b4ed1da.nnue").unwrap().into_raw());
        }

        let mut uci = UCI {
            position: Position::new(None),
            ll_shared_transposition_table: Arc::new(RwLock::new(tt::TranspositionTable::new(2048))),
            is_searching: Arc::new(RwLock::new(false)),
            search_threads: vec![],
            options: Options {
                threads: 1,
                hash: 2048,
            },
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
                    println!("Option name Threads type spin default 1 min 1 max 1024");
                    println!("Option name Hash type spin default 2048 min 1 max 8192");
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

                "setoption" => self.set_option(tokens),

                _ => {
                    println!("Unknown command: {}", buffer.trim());
                }
            }
        }
    }

    fn set_option(&mut self, tokens: Vec<&str>) {
        match tokens.as_slice() {
            [_, "name", name, "value", value] => match *name {
                "Threads" => {
                    let threads = value.parse::<usize>().unwrap();
                    if threads < 1 || threads > 1024 {
                        println!("Threads must be between 1 and 1024");
                        return;
                    }
                    self.options.threads = threads;
                }
                "Hash" => {
                    let hash = value.parse::<usize>().unwrap();
                    if hash < 1 || hash > 8192 {
                        println!("Hash must be between 1 and 8192");
                        return;
                    }
                    self.options.hash = hash;
                    self.ll_shared_transposition_table =
                        Arc::new(RwLock::new(tt::TranspositionTable::new(hash)));
                }
                _ => println!("Unknown option: {}", tokens.get(0).unwrap_or(&"")),
            },
            _ => println!("Unknown option: {}", tokens.get(0).unwrap_or(&"")),
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
        self.is_searching.write().unwrap().clone_from(&false);
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

        let moves = self.position.generate_moves(false);

        let depth = tokens[1].parse::<u8>().unwrap();
        let start_time = std::time::Instant::now();

        let mut nodes = 0;

        for m in moves {
            let is_legal_move = self.position.make_move(m, false);
            if !is_legal_move {
                continue;
            }

            let m_nodes = self.position.perft(depth - 1);
            println!("{}: {}", m, m_nodes);
            nodes += m_nodes;
            self.position.unmake_move();
        }

        let end_time = std::time::Instant::now();
        let elapsed = end_time.duration_since(start_time);
        let nps = nodes as f64 / (elapsed.as_millis() as f64 / 1000.0);
        println!("Nodes searched: {}", nodes);
        println!("Time elapsed (s): {}", elapsed.as_secs_f64());
        println!("Nodes per second: {}", nps);
    }

    /// Start searching with given options
    fn go(&mut self, tokens: Vec<&str>) {
        if tokens.len() == 3 && tokens[1] == "perft" {
            self.perft(tokens[1..].to_vec());
            return;
        }

        let options = SearchOptions::from(tokens);
        let lltt = Arc::clone(&self.ll_shared_transposition_table);
        let is_searching = Arc::clone(&self.is_searching);

        for thread_id in 0..self.options.threads {
            self.search_threads.push(self.create_search_thread(
                self.position.as_fen(),
                options,
                Arc::clone(&is_searching),
                Arc::clone(&lltt),
                thread_id,
            ));
        }
    }

    /// Creates and starts a search thread
    fn create_search_thread(
        &self,
        position_fen: String,
        search_options: SearchOptions,
        is_searching: Arc<RwLock<bool>>,
        ll_transposition_table: Arc<RwLock<tt::TranspositionTable>>,
        thread_id: usize,
    ) -> std::thread::JoinHandle<()> {
        let position_fen_clone = String::from(position_fen.clone());
        let is_searching = Arc::clone(&is_searching);
        let ll_transpos_table_clone = Arc::clone(&ll_transposition_table);

        return std::thread::spawn(move || {
            let mut eval_position = board::Position::new(None);
            eval_position.set_fen(String::from(position_fen_clone));

            let mut evaluator = Evaluator::new();
            evaluator.get_best_move(
                &mut eval_position,
                search_options,
                is_searching,
                ll_transpos_table_clone,
                thread_id,
                1 + thread_id as u8,
            );

            // for i in 0..10 {
            //     println!("{} : {:?}", i, evaluator.nnue_data[i]);
            // }
        });
    }
}
