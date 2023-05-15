use uci::UCI;

mod board;
mod chess;
mod evaluation;
mod movegen;
mod pst;
mod tt;
mod uci;
mod utils;

// #[global_allocator]
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let mut uci = UCI::new();
    uci.uci_loop();

    // for debugging purposes
    // let transposition_table = Arc::new(Mutex::new(tt::TranspositionTable::new()));
    // let running = Arc::new(Mutex::new(true));
    // let mut position = board::Position::new(None);
    // position.set_fen(String::from(chess::constants::STARTING_FEN));

    // println!(
    //     "coun {}",
    //     position.material[chess::Color::White as usize]
    //         - position.material[chess::Color::Black as usize]
    // );

    // let moves = position.generate_legal_moves();
    // for m in moves {
    //     println!("{}", m);
    //     position.make_move(m, false);
    //     position.draw();
    //     println!(
    //         "W {}",
    //         position.material[chess::Color::White as usize]
    //             - position.material[chess::Color::Black as usize]
    //     );
    //     println!(
    //         "B {}",
    //         position.material[chess::Color::Black as usize]
    //             - position.material[chess::Color::White as usize]
    //     );
    //     position.unmake_move();
    //     println!("------------------");
    //     println!(
    //         "W {}",
    //         position.material[chess::Color::White as usize]
    //             - position.material[chess::Color::Black as usize]
    //     );
    //     println!(
    //         "B {}",
    //         position.material[chess::Color::Black as usize]
    //             - position.material[chess::Color::White as usize]
    //     );
    //     println!("------------------");
    //     println!("------------------\n");
    // }

    // let x = "e2e4 e7e5 g1f3 d8f6 b1c3 f6e6 d2d4 c7c5 f3e5 c5d4 d1d4 b8c6 e5c6 b7c6 b2b4 d7d5 c1g5 f7f6 g5e3 c8b7 e4d5 c6d5 f1b5 e8d8 e1g1 e6e3 f2e3 g8e7 a1d1 h7h5 e3e4 f6f5 e4d5 b7c8 d4f4 e7g6 f4g5 g6e7 d5d6 c8d7 d6e7 f8e7 d1d7 d8c8 d7e7 a7a6 g5g7 h8d8 e7b7 a6b5";
    // let toks = Iterator::collect::<Vec<&str>>(x.trim().split_whitespace());

    // for i in 0..toks.len() {
    //     if i % 2 == 0 {
    //         continue;
    //     }
    //     println!("{} {:?}", i, toks[..i].to_vec());
    //     position.set_fen(String::from(chess::constants::STARTING_FEN));
    //     parse_and_make_moves(&mut position, toks[..i].to_vec());

    //     position.draw();

    //     //

    //     let mut options = SearchOptions::new();
    //     options.movetime = Some(5000);

    //     let position_fen = String::from(position.as_fen());

    //     let is_evaluating = Arc::clone(&running);
    //     let transpos_table = Arc::clone(&transposition_table);

    //     let handle: JoinHandle<Option<chess::Move>> = std::thread::spawn(move || {
    //         let mut eval_position = board::Position::new(None);
    //         let mut evaluator = evaluation::Evaluator::new();
    //         eval_position.set_fen(String::from(position_fen));
    //         let best_move =
    //             evaluator.get_best_move(&mut eval_position, options, is_evaluating, transpos_table);
    //         return best_move;
    //     });

    //     handle.join().unwrap();
    // }
}
