use std::{cmp::Ordering, fmt::Display, ops::Not};

use crate::skaak::{self, piece::Piece, square::Square};

pub mod constants {
    pub static STARTING_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub static FILE_A: &'static u64 = &0x0101010101010101;
    // pub static FILE_B: &'static u64 = &0x0202020202020202;
    // pub static FILE_C: &'static u64 = &0x0404040404040404;
    // pub static FILE_D: &'static u64 = &0x0808080808080808;
    // pub static FILE_E: &'static u64 = &0x1010101010101010;
    // pub static FILE_F: &'static u64 = &0x2020202020202020;
    // pub static FILE_G: &'static u64 = &0x4040404040404040;
    pub static FILE_H: &'static u64 = &0x8080808080808080;

    pub static FILE_GH: &'static u64 = &0xC0C0C0C0C0C0C0C0;
    pub static FILE_AB: &'static u64 = &0x0303030303030303;

    pub static RANK_8: &'static u64 = &0x00000000000000FF;
    // pub static RANK_7: &'static u64 = &0x000000000000FF00;
    // pub static RANK_6: &'static u64 = &0x0000000000FF0000;
    // pub static RANK_5: &'static u64 = &0x00000000FF000000;
    // pub static RANK_4: &'static u64 = &0x000000FF00000000;
    // pub static RANK_3: &'static u64 = &0x0000FF0000000000;
    // pub static RANK_2: &'static u64 = &0x00FF000000000000;
    pub static RANK_1: &'static u64 = &0xFF00000000000000;
}
/// Represents the color of a piece
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Color::White => "White",
                Color::Black => "Black",
            }
        )
    }
}

impl From<Color> for usize {
    fn from(v: Color) -> Self {
        match v {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

impl From<char> for Color {
    fn from(v: char) -> Self {
        match v {
            'w' => Color::White,
            'b' => Color::Black,
            _ => panic!("Invalid color"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CastlingRights {
    rights: u8,
}

impl CastlingRights {
    pub const WHITE_KINGSIDE: u8 = 0b0001;
    pub const WHITE_QUEENSIDE: u8 = 0b0010;
    pub const BLACK_KINGSIDE: u8 = 0b0100;
    pub const BLACK_QUEENSIDE: u8 = 0b1000;

    pub fn new() -> Self {
        CastlingRights { rights: 0b1111 }
    }

    pub fn new_empty() -> Self {
        CastlingRights { rights: 0 }
    }

    pub fn can_castle(&self, right: u8) -> bool {
        self.rights & right != 0
    }

    pub fn remove_right(&mut self, right: u8) {
        self.rights &= !right;
    }

    pub fn add_right(&mut self, right: u8) {
        self.rights |= right;
    }

    pub fn get_rights_u8(&self) -> u8 {
        self.rights
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub capture: Option<Piece>,
    pub castle: bool,
    pub promotion: Option<Piece>,
    pub en_passant: bool,
}

impl From<&str> for CastlingRights {
    fn from(v: &str) -> Self {
        let mut castling_rights = CastlingRights::new_empty();
        for c in v.chars() {
            match c {
                'K' => castling_rights.add_right(CastlingRights::WHITE_KINGSIDE),
                'Q' => castling_rights.add_right(CastlingRights::WHITE_QUEENSIDE),
                'k' => castling_rights.add_right(CastlingRights::BLACK_KINGSIDE),
                'q' => castling_rights.add_right(CastlingRights::BLACK_QUEENSIDE),
                '-' => break,
                _ => panic!("Invalid castling rights"),
            }
        }
        return castling_rights;
    }
}

pub struct PrioritizedMove {
    pub priority: u32,
    pub m: skaak::_move::BitPackedMove,
}

impl PartialEq for PrioritizedMove {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedMove {}

impl Ord for PrioritizedMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for PrioritizedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
