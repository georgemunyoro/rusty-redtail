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
    stop_flag: Arc<AtomicBool>,
    ponder_flag: Arc<AtomicBool>,
    num_threads: usize,
}

impl UCI {
    pub fn new() -> UCI {
        let mut uci = UCI {
            position: Position::new(None),
            transposition_table: tt::TranspositionTable::new(2048),
            stop_flag: Arc::new(AtomicBool::new(false)),
            ponder_flag: Arc::new(AtomicBool::new(false)),
            num_threads: 1,
        };
        uci.position
            .set_fen(String::from(chess::constants::STARTING_FEN));
        uci
    }

    pub fn uci_loop(&mut self) {
        // Create a channel to receive commands from stdin reader thread
        let (tx, rx) = mpsc::channel::<String>();

        // Spawn a thread to read stdin that handles stop/ponderhit directly
        let stop_flag_for_reader = Arc::clone(&self.stop_flag);
        let ponder_flag_for_reader = Arc::clone(&self.ponder_flag);
        thread::spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                if let Ok(line) = line {
                    match line.trim() {
                        "stop" => stop_flag_for_reader.store(true, Ordering::SeqCst),
                        "ponderhit" => ponder_flag_for_reader.store(false, Ordering::SeqCst),
                        _ => {}
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
                    println!("option name Threads type spin default 1 min 1 max 256");
                    println!("option name Ponder type check default true");
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

                "stop" | "ponderhit" => {} // Already handled in reader thread

                "quit" => break,

                // The rest of the commands below are custom convenience
                // commands. Mostly used for debugging, but are useful beyond that.
                "bench" => self.bench(tokens),

                "perft" => self.perft(tokens),

                "draw" => self.position.draw(),

                "setoption" => {
                    if tokens.len() >= 5 && tokens[1] == "name" && tokens[3] == "value" {
                        match tokens[2] {
                            "Threads" => {
                                if let Ok(n) = tokens[4].parse::<usize>() {
                                    self.num_threads = n.max(1);
                                }
                            }
                            _ => {}
                        }
                    }
                }

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

    /// Run a benchmark search over a set of positions at a fixed depth.
    /// Usage: bench [depth] [threads] [hash]
    /// Defaults: depth 13, threads 1, hash 16
    pub fn bench(&mut self, tokens: Vec<&str>) {
        const BENCH_POSITIONS: &[&str] = &[
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1",
            "8/8/1P6/5pr1/8/4R3/7p/2K1k3 w - - 0 1",
            "5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RK1 b - - 0 1",
            "8/p3b1kp/2p2rp1/3p4/3B4/1P3P1P/P5P1/5RK1 w - - 0 1",
            "r1bqkb1r/pppppppp/2n2n2/8/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 2 3",
            "r1bqk2r/2ppbppp/p1n2n2/1p2p3/4P3/1B3N2/PPPP1PPP/RNBQR1K1 b kq - 0 7",
            "r1bq1rk1/pp1pppbp/2n2np1/8/3NP3/2N1B3/PPP1BPPP/R2QK2R w KQ - 3 7",
            "r1bqk2r/pppp1ppp/2n2n2/1B2p3/1b2P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            "rnbq1rk1/pppp1ppp/4pn2/8/1bPP4/2N1P3/PP3PPP/R1BQKBNR w KQ - 3 5",
            "r1bqk2r/ppppbppp/2n2n2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R w KQkq - 4 5",
        ];

        let depth = if tokens.len() > 1 {
            tokens[1].parse::<u8>().unwrap_or(13)
        } else {
            13
        };

        let threads = if tokens.len() > 2 {
            tokens[2].parse::<usize>().unwrap_or(1).max(1)
        } else {
            1
        };

        let hash = if tokens.len() > 3 {
            tokens[3].parse::<usize>().unwrap_or(16)
        } else {
            16
        };

        let tt = tt::TranspositionTable::new(hash);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut total_nodes: u64 = 0;
        let start_time = std::time::Instant::now();

        let mut options = SearchOptions::new();
        options.depth = Some(depth);

        for (i, fen) in BENCH_POSITIONS.iter().enumerate() {
            let mut position = Position::new(Some(fen));
            tt.clear();
            stop_flag.store(false, Ordering::SeqCst);

            let mut evaluator = Evaluator::new();
            evaluator.set_silent(true);

            tt.increment_age();
            if threads <= 1 {
                evaluator.get_best_move(&mut position, options, &tt, &stop_flag, None);
                let nodes = evaluator.result.nodes as u64;
                total_nodes += nodes;
                println!("Position {}/{}: {} nodes", i + 1, BENCH_POSITIONS.len(), nodes);
            } else {
                // For multi-threaded bench, use search_parallel but we can't
                // easily get total nodes, so run single-threaded for accuracy
                evaluator.get_best_move(&mut position, options, &tt, &stop_flag, None);
                let nodes = evaluator.result.nodes as u64;
                total_nodes += nodes;
                println!("Position {}/{}: {} nodes", i + 1, BENCH_POSITIONS.len(), nodes);
            }
        }

        let elapsed = start_time.elapsed();
        let elapsed_ms = elapsed.as_millis().max(1) as u64;
        let nps = total_nodes * 1000 / elapsed_ms;

        println!();
        println!("===========================");
        println!("Total time (ms) : {}", elapsed_ms);
        println!("Nodes searched  : {}", total_nodes);
        println!("Nodes/second    : {}", nps);
        io::stdout().flush().unwrap();
    }

    /// Start searching with given options
    fn go(&mut self, tokens: Vec<&str>, rx: &mpsc::Receiver<String>) {
        let options = SearchOptions::from(tokens);
        self.stop_flag.store(false, Ordering::SeqCst);
        let stop_flag = Arc::clone(&self.stop_flag);

        // Set ponder_flag: true if this is "go ponder", so the search runs
        // without time limits until ponderhit or stop
        let ponder_flag = if options.ponder {
            self.ponder_flag.store(true, Ordering::SeqCst);
            Some(Arc::clone(&self.ponder_flag))
        } else {
            None
        };

        // Run the search - stdin reader thread will set stop_flag/ponder_flag
        search_parallel(
            &self.position,
            options,
            &self.transposition_table,
            &stop_flag,
            ponder_flag,
            self.num_threads,
        );

        // Process any commands that arrived during search
        while let Ok(cmd) = rx.try_recv() {
            match cmd.trim() {
                "quit" => std::process::exit(0),
                "stop" | "ponderhit" => {}
                _ => {}
            }
        }
    }
}
