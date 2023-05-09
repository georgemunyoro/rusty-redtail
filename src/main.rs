mod board;
mod chess;
mod evaluation;
mod movegen;
mod uci;
mod utils;
use crate::board::*;

fn main() {
    let mut uci = uci::UCI::new();
    uci.uci_loop();
    // let mut main_board =
    //     board::Position::new(Some(String::from(chess::constants::STARTING_FEN).as_str()));
    // main_board.debug();
}
