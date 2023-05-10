use std::{fmt::Display, ops::Not};

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

/// Represents a square on the chess board
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd)]
pub enum Square {
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    NoSq,
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Square::A8 => write!(f, "A8"),
            Square::B8 => write!(f, "B8"),
            Square::C8 => write!(f, "C8"),
            Square::D8 => write!(f, "D8"),
            Square::E8 => write!(f, "E8"),
            Square::F8 => write!(f, "F8"),
            Square::G8 => write!(f, "G8"),
            Square::H8 => write!(f, "H8"),
            Square::A7 => write!(f, "A7"),
            Square::B7 => write!(f, "B7"),
            Square::C7 => write!(f, "C7"),
            Square::D7 => write!(f, "D7"),
            Square::E7 => write!(f, "E7"),
            Square::F7 => write!(f, "F7"),
            Square::G7 => write!(f, "G7"),
            Square::H7 => write!(f, "H7"),
            Square::A6 => write!(f, "A6"),
            Square::B6 => write!(f, "B6"),
            Square::C6 => write!(f, "C6"),
            Square::D6 => write!(f, "D6"),
            Square::E6 => write!(f, "E6"),
            Square::F6 => write!(f, "F6"),
            Square::G6 => write!(f, "G6"),
            Square::H6 => write!(f, "H6"),
            Square::A5 => write!(f, "A5"),
            Square::B5 => write!(f, "B5"),
            Square::C5 => write!(f, "C5"),
            Square::D5 => write!(f, "D5"),
            Square::E5 => write!(f, "E5"),
            Square::F5 => write!(f, "F5"),
            Square::G5 => write!(f, "G5"),
            Square::H5 => write!(f, "H5"),
            Square::A4 => write!(f, "A4"),
            Square::B4 => write!(f, "B4"),
            Square::C4 => write!(f, "C4"),
            Square::D4 => write!(f, "D4"),
            Square::E4 => write!(f, "E4"),
            Square::F4 => write!(f, "F4"),
            Square::G4 => write!(f, "G4"),
            Square::H4 => write!(f, "H4"),
            Square::A3 => write!(f, "A3"),
            Square::B3 => write!(f, "B3"),
            Square::C3 => write!(f, "C3"),
            Square::D3 => write!(f, "D3"),
            Square::E3 => write!(f, "E3"),
            Square::F3 => write!(f, "F3"),
            Square::G3 => write!(f, "G3"),
            Square::H3 => write!(f, "H3"),
            Square::A2 => write!(f, "A2"),
            Square::B2 => write!(f, "B2"),
            Square::C2 => write!(f, "C2"),
            Square::D2 => write!(f, "D2"),
            Square::E2 => write!(f, "E2"),
            Square::F2 => write!(f, "F2"),
            Square::G2 => write!(f, "G2"),
            Square::H2 => write!(f, "H2"),
            Square::A1 => write!(f, "A1"),
            Square::B1 => write!(f, "B1"),
            Square::C1 => write!(f, "C1"),
            Square::D1 => write!(f, "D1"),
            Square::E1 => write!(f, "E1"),
            Square::F1 => write!(f, "F1"),
            Square::G1 => write!(f, "G1"),
            Square::H1 => write!(f, "H1"),
            Square::NoSq => write!(f, "NoSq"),
        }
    }
}

pub static SQUARE_ITER: [Square; 64] = [
    Square::A8,
    Square::B8,
    Square::C8,
    Square::D8,
    Square::E8,
    Square::F8,
    Square::G8,
    Square::H8,
    Square::A7,
    Square::B7,
    Square::C7,
    Square::D7,
    Square::E7,
    Square::F7,
    Square::G7,
    Square::H7,
    Square::A6,
    Square::B6,
    Square::C6,
    Square::D6,
    Square::E6,
    Square::F6,
    Square::G6,
    Square::H6,
    Square::A5,
    Square::B5,
    Square::C5,
    Square::D5,
    Square::E5,
    Square::F5,
    Square::G5,
    Square::H5,
    Square::A4,
    Square::B4,
    Square::C4,
    Square::D4,
    Square::E4,
    Square::F4,
    Square::G4,
    Square::H4,
    Square::A3,
    Square::B3,
    Square::C3,
    Square::D3,
    Square::E3,
    Square::F3,
    Square::G3,
    Square::H3,
    Square::A2,
    Square::B2,
    Square::C2,
    Square::D2,
    Square::E2,
    Square::F2,
    Square::G2,
    Square::H2,
    Square::A1,
    Square::B1,
    Square::C1,
    Square::D1,
    Square::E1,
    Square::F1,
    Square::G1,
    Square::H1,
];

impl From<usize> for Square {
    fn from(square: usize) -> Square {
        match square {
            0 => Square::A8,
            1 => Square::B8,
            2 => Square::C8,
            3 => Square::D8,
            4 => Square::E8,
            5 => Square::F8,
            6 => Square::G8,
            7 => Square::H8,
            8 => Square::A7,
            9 => Square::B7,
            10 => Square::C7,
            11 => Square::D7,
            12 => Square::E7,
            13 => Square::F7,
            14 => Square::G7,
            15 => Square::H7,
            16 => Square::A6,
            17 => Square::B6,
            18 => Square::C6,
            19 => Square::D6,
            20 => Square::E6,
            21 => Square::F6,
            22 => Square::G6,
            23 => Square::H6,
            24 => Square::A5,
            25 => Square::B5,
            26 => Square::C5,
            27 => Square::D5,
            28 => Square::E5,
            29 => Square::F5,
            30 => Square::G5,
            31 => Square::H5,
            32 => Square::A4,
            33 => Square::B4,
            34 => Square::C4,
            35 => Square::D4,
            36 => Square::E4,
            37 => Square::F4,
            38 => Square::G4,
            39 => Square::H4,
            40 => Square::A3,
            41 => Square::B3,
            42 => Square::C3,
            43 => Square::D3,
            44 => Square::E3,
            45 => Square::F3,
            46 => Square::G3,
            47 => Square::H3,
            48 => Square::A2,
            49 => Square::B2,
            50 => Square::C2,
            51 => Square::D2,
            52 => Square::E2,
            53 => Square::F2,
            54 => Square::G2,
            55 => Square::H2,
            56 => Square::A1,
            57 => Square::B1,
            58 => Square::C1,
            59 => Square::D1,
            60 => Square::E1,
            61 => Square::F1,
            62 => Square::G1,
            63 => Square::H1,
            _ => Square::NoSq,
        }
    }
}

impl From<u8> for Square {
    fn from(square: u8) -> Square {
        match square {
            0 => Square::A8,
            1 => Square::B8,
            2 => Square::C8,
            3 => Square::D8,
            4 => Square::E8,
            5 => Square::F8,
            6 => Square::G8,
            7 => Square::H8,
            8 => Square::A7,
            9 => Square::B7,
            10 => Square::C7,
            11 => Square::D7,
            12 => Square::E7,
            13 => Square::F7,
            14 => Square::G7,
            15 => Square::H7,
            16 => Square::A6,
            17 => Square::B6,
            18 => Square::C6,
            19 => Square::D6,
            20 => Square::E6,
            21 => Square::F6,
            22 => Square::G6,
            23 => Square::H6,
            24 => Square::A5,
            25 => Square::B5,
            26 => Square::C5,
            27 => Square::D5,
            28 => Square::E5,
            29 => Square::F5,
            30 => Square::G5,
            31 => Square::H5,
            32 => Square::A4,
            33 => Square::B4,
            34 => Square::C4,
            35 => Square::D4,
            36 => Square::E4,
            37 => Square::F4,
            38 => Square::G4,
            39 => Square::H4,
            40 => Square::A3,
            41 => Square::B3,
            42 => Square::C3,
            43 => Square::D3,
            44 => Square::E3,
            45 => Square::F3,
            46 => Square::G3,
            47 => Square::H3,
            48 => Square::A2,
            49 => Square::B2,
            50 => Square::C2,
            51 => Square::D2,
            52 => Square::E2,
            53 => Square::F2,
            54 => Square::G2,
            55 => Square::H2,
            56 => Square::A1,
            57 => Square::B1,
            58 => Square::C1,
            59 => Square::D1,
            60 => Square::E1,
            61 => Square::F1,
            62 => Square::G1,
            63 => Square::H1,
            _ => Square::NoSq,
        }
    }
}

impl From<Square> for u8 {
    fn from(square: Square) -> u8 {
        match square {
            Square::A8 => 0,
            Square::B8 => 1,
            Square::C8 => 2,
            Square::D8 => 3,
            Square::E8 => 4,
            Square::F8 => 5,
            Square::G8 => 6,
            Square::H8 => 7,
            Square::A7 => 8,
            Square::B7 => 9,
            Square::C7 => 10,
            Square::D7 => 11,
            Square::E7 => 12,
            Square::F7 => 13,
            Square::G7 => 14,
            Square::H7 => 15,
            Square::A6 => 16,
            Square::B6 => 17,
            Square::C6 => 18,
            Square::D6 => 19,
            Square::E6 => 20,
            Square::F6 => 21,
            Square::G6 => 22,
            Square::H6 => 23,
            Square::A5 => 24,
            Square::B5 => 25,
            Square::C5 => 26,
            Square::D5 => 27,
            Square::E5 => 28,
            Square::F5 => 29,
            Square::G5 => 30,
            Square::H5 => 31,
            Square::A4 => 32,
            Square::B4 => 33,
            Square::C4 => 34,
            Square::D4 => 35,
            Square::E4 => 36,
            Square::F4 => 37,
            Square::G4 => 38,
            Square::H4 => 39,
            Square::A3 => 40,
            Square::B3 => 41,
            Square::C3 => 42,
            Square::D3 => 43,
            Square::E3 => 44,
            Square::F3 => 45,
            Square::G3 => 46,
            Square::H3 => 47,
            Square::A2 => 48,
            Square::B2 => 49,
            Square::C2 => 50,
            Square::D2 => 51,
            Square::E2 => 52,
            Square::F2 => 53,
            Square::G2 => 54,
            Square::H2 => 55,
            Square::A1 => 56,
            Square::B1 => 57,
            Square::C1 => 58,
            Square::D1 => 59,
            Square::E1 => 60,
            Square::F1 => 61,
            Square::G1 => 62,
            Square::H1 => 63,
            Square::NoSq => 64,
        }
    }
}

impl From<&str> for Square {
    fn from(square: &str) -> Square {
        match square {
            "a8" => Square::A8,
            "b8" => Square::B8,
            "c8" => Square::C8,
            "d8" => Square::D8,
            "e8" => Square::E8,
            "f8" => Square::F8,
            "g8" => Square::G8,
            "h8" => Square::H8,
            "a7" => Square::A7,
            "b7" => Square::B7,
            "c7" => Square::C7,
            "d7" => Square::D7,
            "e7" => Square::E7,
            "f7" => Square::F7,
            "g7" => Square::G7,
            "h7" => Square::H7,
            "a6" => Square::A6,
            "b6" => Square::B6,
            "c6" => Square::C6,
            "d6" => Square::D6,
            "e6" => Square::E6,
            "f6" => Square::F6,
            "g6" => Square::G6,
            "h6" => Square::H6,
            "a5" => Square::A5,
            "b5" => Square::B5,
            "c5" => Square::C5,
            "d5" => Square::D5,
            "e5" => Square::E5,
            "f5" => Square::F5,
            "g5" => Square::G5,
            "h5" => Square::H5,
            "a4" => Square::A4,
            "b4" => Square::B4,
            "c4" => Square::C4,
            "d4" => Square::D4,
            "e4" => Square::E4,
            "f4" => Square::F4,
            "g4" => Square::G4,
            "h4" => Square::H4,
            "a3" => Square::A3,
            "b3" => Square::B3,
            "c3" => Square::C3,
            "d3" => Square::D3,
            "e3" => Square::E3,
            "f3" => Square::F3,
            "g3" => Square::G3,
            "h3" => Square::H3,
            "a2" => Square::A2,
            "b2" => Square::B2,
            "c2" => Square::C2,
            "d2" => Square::D2,
            "e2" => Square::E2,
            "f2" => Square::F2,
            "g2" => Square::G2,
            "h2" => Square::H2,
            "a1" => Square::A1,
            "b1" => Square::B1,
            "c1" => Square::C1,
            "d1" => Square::D1,
            "e1" => Square::E1,
            "f1" => Square::F1,
            "g1" => Square::G1,
            "h1" => Square::H1,
            _ => Square::NoSq,
        }
    }
}

impl From<Square> for i8 {
    fn from(square: Square) -> i8 {
        match square {
            Square::A8 => 0,
            Square::B8 => 1,
            Square::C8 => 2,
            Square::D8 => 3,
            Square::E8 => 4,
            Square::F8 => 5,
            Square::G8 => 6,
            Square::H8 => 7,
            Square::A7 => 8,
            Square::B7 => 9,
            Square::C7 => 10,
            Square::D7 => 11,
            Square::E7 => 12,
            Square::F7 => 13,
            Square::G7 => 14,
            Square::H7 => 15,
            Square::A6 => 16,
            Square::B6 => 17,
            Square::C6 => 18,
            Square::D6 => 19,
            Square::E6 => 20,
            Square::F6 => 21,
            Square::G6 => 22,
            Square::H6 => 23,
            Square::A5 => 24,
            Square::B5 => 25,
            Square::C5 => 26,
            Square::D5 => 27,
            Square::E5 => 28,
            Square::F5 => 29,
            Square::G5 => 30,
            Square::H5 => 31,
            Square::A4 => 32,
            Square::B4 => 33,
            Square::C4 => 34,
            Square::D4 => 35,
            Square::E4 => 36,
            Square::F4 => 37,
            Square::G4 => 38,
            Square::H4 => 39,
            Square::A3 => 40,
            Square::B3 => 41,
            Square::C3 => 42,
            Square::D3 => 43,
            Square::E3 => 44,
            Square::F3 => 45,
            Square::G3 => 46,
            Square::H3 => 47,
            Square::A2 => 48,
            Square::B2 => 49,
            Square::C2 => 50,
            Square::D2 => 51,
            Square::E2 => 52,
            Square::F2 => 53,
            Square::G2 => 54,
            Square::H2 => 55,
            Square::A1 => 56,
            Square::B1 => 57,
            Square::C1 => 58,
            Square::D1 => 59,
            Square::E1 => 60,
            Square::F1 => 61,
            Square::G1 => 62,
            Square::H1 => 63,
            Square::NoSq => 64,
        }
    }
}

/// Represents a chess piece
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Piece {
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,

    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,

    Empty,
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
            Piece::BlackPawn => 0,
            Piece::BlackKnight => 1,
            Piece::BlackBishop => 2,
            Piece::BlackRook => 3,
            Piece::BlackQueen => 4,
            Piece::BlackKing => 5,
            Piece::WhitePawn => 6,
            Piece::WhiteKnight => 7,
            Piece::WhiteBishop => 8,
            Piece::WhiteRook => 9,
            Piece::WhiteQueen => 10,
            Piece::WhiteKing => 11,
            Piece::Empty => 12,
        }
    }
}

impl From<usize> for Piece {
    fn from(v: usize) -> Self {
        match v {
            0 => Piece::BlackPawn,
            1 => Piece::BlackKnight,
            2 => Piece::BlackBishop,
            3 => Piece::BlackRook,
            4 => Piece::BlackQueen,
            5 => Piece::BlackKing,
            6 => Piece::WhitePawn,
            7 => Piece::WhiteKnight,
            8 => Piece::WhiteBishop,
            9 => Piece::WhiteRook,
            10 => Piece::WhiteQueen,
            11 => Piece::WhiteKing,
            12 => Piece::Empty,
            _ => panic!("Invalid piece"),
        }
    }
}

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

/// Represents the color of a piece
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White,
    Black,
    Both,
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
            Color::Both => Color::Both,
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
                Color::Both => "Both",
            }
        )
    }
}

impl From<Color> for usize {
    fn from(v: Color) -> Self {
        match v {
            Color::White => 0,
            Color::Black => 1,
            Color::Both => 2,
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
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub capture: Option<Piece>,
    pub castle: bool,
    pub promotion: Option<Piece>,
    pub en_passant: bool,
}

impl Move {
    pub fn new(from: Square, to: Square, piece: Piece) -> Self {
        Move {
            from,
            to,
            piece,
            capture: None,
            castle: false,
            promotion: None,
            en_passant: false,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // return the move in long uci notation
        write!(
            f,
            "{}{}{}",
            self.from.to_string().to_lowercase(),
            self.to.to_string().to_lowercase(),
            match self.promotion {
                Some(p) => get_piece_char(p).to_lowercase().to_string(),
                None => "".to_string(),
            },
        )
    }
}

impl Move {
    /// Returns the move in SAN notation
    pub fn as_san(&self) -> String {
        let piece_symbol = match self.piece {
            Piece::WhitePawn => "".to_string(),
            Piece::BlackPawn => "".to_string(),
            _ => get_piece_char(self.piece).to_string().to_uppercase(),
        };

        let capture_symbol = match self.capture {
            Some(_) => "x",
            None => "",
        };

        return format!(
            "{}{}{}",
            piece_symbol,
            capture_symbol,
            self.to.to_string().to_lowercase(),
        );
    }
}

impl From<&str> for CastlingRights {
    fn from(v: &str) -> Self {
        let mut white_king_side = false;
        let mut white_queen_side = false;
        let mut black_king_side = false;
        let mut black_queen_side = false;

        for c in v.chars() {
            match c {
                'K' => white_king_side = true,
                'Q' => white_queen_side = true,
                'k' => black_king_side = true,
                'q' => black_queen_side = true,
                '-' => break,
                _ => panic!("Invalid castling rights"),
            }
        }

        return CastlingRights {
            white_king_side,
            white_queen_side,
            black_king_side,
            black_queen_side,
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::chess::{get_piece_char, Piece};

    #[test]
    fn get_piece_char_gets_the_correct_character() {
        assert_eq!(get_piece_char(Piece::BlackPawn), 'p');
        assert_eq!(get_piece_char(Piece::BlackKnight), 'n');
        assert_eq!(get_piece_char(Piece::BlackBishop), 'b');
        assert_eq!(get_piece_char(Piece::BlackRook), 'r');
        assert_eq!(get_piece_char(Piece::BlackQueen), 'q');
        assert_eq!(get_piece_char(Piece::BlackKing), 'k');
        assert_eq!(get_piece_char(Piece::WhitePawn), 'P');
        assert_eq!(get_piece_char(Piece::WhiteKnight), 'N');
        assert_eq!(get_piece_char(Piece::WhiteBishop), 'B');
        assert_eq!(get_piece_char(Piece::WhiteRook), 'R');
        assert_eq!(get_piece_char(Piece::WhiteQueen), 'Q');
        assert_eq!(get_piece_char(Piece::WhiteKing), 'K');
        assert_eq!(get_piece_char(Piece::Empty), '.');
    }
}
