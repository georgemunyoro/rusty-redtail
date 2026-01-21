use std::{
    io::{self, BufRead, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use redtail::{
    board::{Board, Position},
    chess,
    movegen::MoveGenerator,
    search::evaluate::*,
    search::options::*,
    search::utils::*,
    tt,
};

pub struct UCI {
    position: Position,
    transposition_table: tt::TranspositionTable,
    evaluator: Evaluator,
    stop_flag: Arc<AtomicBool>,
}

impl UCI {
    pub fn new() -> UCI {
        let mut uci = UCI {
            position: Position::new(None),
            transposition_table: tt::TranspositionTable::new(2048),
            evaluator: Evaluator::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
        };
        uci.position
            .set_fen(String::from(chess::constants::STARTING_FEN));
        uci
    }

    pub fn uci_loop(&mut self) {
        // Create a channel to receive commands from stdin reader thread
        let (tx, rx) = mpsc::channel::<String>();

        // Spawn a thread to read stdin that also handles stop directly
        let stop_flag_for_reader = Arc::clone(&self.stop_flag);
        thread::spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                if let Ok(line) = line {
                    // Handle stop immediately by setting the flag
                    if line.trim() == "stop" {
                        stop_flag_for_reader.store(true, Ordering::SeqCst);
                    }
                    if tx.send(line).is_err() {
                        break;
                    }
                }
            }
        });

        loop {
            let buffer = match rx.recv() {
                Ok(line) => line,
                Err(_) => break,
            };

            let tokens = Iterator::collect::<Vec<&str>>(buffer.trim().split_whitespace());

            if tokens.is_empty() {
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
                    println!("option name Hash type spin default 1 min 1 max 1");
                    println!("uciok");
                    io::stdout().flush().unwrap();
                }

                "isready" => {
                    println!("readyok");
                    io::stdout().flush().unwrap();
                }

                "ucinewgame" => {
                    self.position
                        .set_fen(String::from(chess::constants::STARTING_FEN));
                    self.transposition_table.clear();
                }

                "position" => self.handle_position(tokens),

                "go" => self.go(tokens, &rx),

                "stop" => {} // Already handled in reader thread

                "quit" => break,

                // The rest of the commands below are custom convenience
                // commands. Mostly used for debugging, but are useful beyond that.
                "perft" => self.perft(tokens),

                "draw" => self.position.draw(),

                "setoption" => {}

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
    fn go(&mut self, tokens: Vec<&str>, rx: &mpsc::Receiver<String>) {
        let options = SearchOptions::from(tokens);
        self.stop_flag.store(false, Ordering::SeqCst);
        let stop_flag = Arc::clone(&self.stop_flag);

        // Run the search - stdin reader thread will set stop_flag if "stop" is received
        self.evaluator.get_best_move(
            &mut self.position,
            options,
            &mut self.transposition_table,
            &stop_flag,
        );

        // Process any commands that arrived during search
        while let Ok(cmd) = rx.try_recv() {
            match cmd.trim() {
                "quit" => std::process::exit(0),
                _ => {}
            }
        }
    }
}
