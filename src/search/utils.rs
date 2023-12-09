use crate::{
    board::{self},
    chess::{self},
    movegen::MoveGenerator,
};

/// Parses a uci move string and returns a valid move struct
pub fn parse_move(
    position: &mut board::Position,
    move_string: &str,
) -> Option<chess::_move::BitPackedMove> {
    let moves = position.generate_moves(false);
    for m in moves {
        if m.to_string() == move_string {
            return Some(m);
        }
    }
    return None;
}
