use std::fmt::Display;

use super::color::Color;

/// Represents a chess piece
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,

    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,

    Empty,
}

impl Piece {
    pub fn pawn(color: Color) -> Piece {
        return match color {
            Color::White => Piece::WhitePawn,
            Color::Black => Piece::BlackPawn,
        };
    }

    pub fn knight(color: Color) -> Piece {
        return match color {
            Color::White => Piece::WhiteKnight,
            Color::Black => Piece::BlackKnight,
        };
    }

    pub fn bishop(color: Color) -> Piece {
        return match color {
            Color::White => Piece::WhiteBishop,
            Color::Black => Piece::BlackBishop,
        };
    }

    pub fn rook(color: Color) -> Piece {
        return match color {
            Color::White => Piece::WhiteRook,
            Color::Black => Piece::BlackRook,
        };
    }

    pub fn queen(color: Color) -> Piece {
        return match color {
            Color::White => Piece::WhiteQueen,
            Color::Black => Piece::BlackQueen,
        };
    }

    pub fn king(color: Color) -> Piece {
        return match color {
            Color::White => Piece::WhiteKing,
            Color::Black => Piece::BlackKing,
        };
    }

    pub fn is_pawn(self) -> bool {
        return self == Piece::WhitePawn || self == Piece::BlackPawn;
    }

    pub fn is_king(self) -> bool {
        return self == Piece::WhiteKing || self == Piece::BlackKing;
    }

    pub fn is_rook(self) -> bool {
        return self == Piece::WhiteRook || self == Piece::BlackRook;
    }
}

/// Returns the character representation of a piece
fn get_piece_char(piece: Piece) -> char {
    match piece {
        Piece::BlackPawn => 'p',
        Piece::BlackKnight => 'n',
        Piece::BlackBishop => 'b',
        Piece::BlackRook => 'r',
        Piece::BlackQueen => 'q',
        Piece::BlackKing => 'k',
        Piece::WhitePawn => 'P',
        Piece::WhiteKnight => 'N',
        Piece::WhiteBishop => 'B',
        Piece::WhiteRook => 'R',
        Piece::WhiteQueen => 'Q',
        Piece::WhiteKing => 'K',
        Piece::Empty => '.',
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_piece_char(*self))
    }
}

impl Into<char> for Piece {
    fn into(self) -> char {
        get_piece_char(self)
    }
}

impl From<char> for Piece {
    fn from(v: char) -> Self {
        match v {
            'p' => Piece::BlackPawn,
            'n' => Piece::BlackKnight,
            'b' => Piece::BlackBishop,
            'r' => Piece::BlackRook,
            'q' => Piece::BlackQueen,
            'k' => Piece::BlackKing,
            'P' => Piece::WhitePawn,
            'N' => Piece::WhiteKnight,
            'B' => Piece::WhiteBishop,
            'R' => Piece::WhiteRook,
            'Q' => Piece::WhiteQueen,
            'K' => Piece::WhiteKing,
            '.' => Piece::Empty,
            _ => panic!("Invalid piece"),
        }
    }
}

impl From<Piece> for usize {
    fn from(v: Piece) -> Self {
        match v {
            Piece::WhitePawn => 0,
            Piece::WhiteKnight => 1,
            Piece::WhiteBishop => 2,
            Piece::WhiteRook => 3,
            Piece::WhiteQueen => 4,
            Piece::WhiteKing => 5,
            Piece::BlackPawn => 6,
            Piece::BlackKnight => 7,
            Piece::BlackBishop => 8,
            Piece::BlackRook => 9,
            Piece::BlackQueen => 10,
            Piece::BlackKing => 11,
            Piece::Empty => 12,
        }
    }
}

impl From<usize> for Piece {
    fn from(v: usize) -> Self {
        match v {
            0 => Piece::WhitePawn,
            1 => Piece::WhiteKnight,
            2 => Piece::WhiteBishop,
            3 => Piece::WhiteRook,
            4 => Piece::WhiteQueen,
            5 => Piece::WhiteKing,
            6 => Piece::BlackPawn,
            7 => Piece::BlackKnight,
            8 => Piece::BlackBishop,
            9 => Piece::BlackRook,
            10 => Piece::BlackQueen,
            11 => Piece::BlackKing,
            12 => Piece::Empty,
            _ => panic!("Invalid piece"),
        }
    }
}

/// An convenience array for looping over pieces
/// from black pawn -> white king
pub static PIECE_ITER: [Piece; 12] = [
    Piece::BlackPawn,
    Piece::BlackKnight,
    Piece::BlackBishop,
    Piece::BlackRook,
    Piece::BlackQueen,
    Piece::BlackKing,
    Piece::WhitePawn,
    Piece::WhiteKnight,
    Piece::WhiteBishop,
    Piece::WhiteRook,
    Piece::WhiteQueen,
    Piece::WhiteKing,
];
