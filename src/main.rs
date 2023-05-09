mod board;
mod chess;
mod movegen;
mod utils;
use crate::board::*;

fn main() {
    let mut main_board =
        board::Position::new(Some(String::from(chess::constants::STARTING_FEN).as_str()));
    main_board.debug();
}
