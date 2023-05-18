use uci::UCI;

mod board;
mod chess;
mod movegen;
mod pst;
mod search;
mod tt;
mod uci;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let mut uci = UCI::new();
    uci.uci_loop();

    //
    // let mut position = board::Position::new(None);
    // let mut evaluator = search::evaluate::Evaluator::new();
    // position.set_fen(String::from(
    //     // "4rr2/1p4bk/2p3pn/B3n2b/P4N1q/1P5P/6PK/1BQ1RR2 b - - 1 31",
    //     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    // ));
    // // println!("{}", evaluator.evaluate(&mut position));
    // // println!("{}", position.get_game_phase_score());

    // let mut black = 0;
    // let mut white = 0;

    // for i in 0..64 {
    //     let piece = position.get_piece_at_square(i);
    //     if piece == chess::piece::Piece::Empty {
    //         continue;
    //     }
    //     let v = _get_piece_value(piece, i as usize, position.get_game_phase_score());
    //     println!("{}: {} : {}", i, piece, v);
    //     if piece as usize >= 6 {
    //         black += v;
    //     } else {
    //         white += v;
    //     }
    // }

    // println!("White: {}", white);
    // println!("Black: {}", black);
    // println!("evaluate: {}", evaluator.evaluate(&mut position));
}
