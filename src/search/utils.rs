use crate::{
    board::{self, Board},
    chess,
    movegen::MoveGenerator,
};

/// Parses a vector of moves and makes them on the given board
pub fn parse_and_make_moves(position: &mut board::Position, moves: Vec<&str>) {
    for m in moves {
        if let Some(parsed_move) = parse_move(position, m) {
            position.make_move(&parsed_move, false);
        };
    }
}

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
