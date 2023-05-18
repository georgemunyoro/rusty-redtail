use crate::chess::square::Square;

pub static ROOK_POSITIONAL_SCORE: [i32; 64] = [
    50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 0, 0, 10, 20, 20, 10, 0, 0, 0,
    0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20,
    20, 10, 0, 0, 0, 0, 0, 20, 20, 0, 0, 0,
];

pub static KNIGHT_POSITIONAL_SCORE: [i32; 64] = [
    -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 10, 10, 0, 0, -5, -5, 5, 20, 20, 20, 20, 5, -5, -5, 10, 20,
    30, 30, 20, 10, -5, -5, 10, 20, 30, 30, 20, 10, -5, -5, 5, 20, 10, 10, 20, 5, -5, -5, 0, 0, 0,
    0, 0, 0, -5, -5, -10, 0, 0, 0, 0, -10, -5,
];

pub static BISHOP_POSITIONAL_SCORE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 0, 0, 0, 0, 0, 10, 20, 20, 10,
    0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 10, 0, 0, 0, 0, 10, 0, 0, 30, 0, 0, 0, 0, 30, 0, 0, 0,
    -10, 0, 0, -10, 0, 0,
];

pub static PAWN_POSITIONAL_SCORE: [i32; 64] = [
    90, 90, 90, 90, 90, 90, 90, 90, 30, 30, 30, 40, 40, 30, 30, 30, 20, 20, 20, 30, 30, 30, 20, 20,
    10, 10, 10, 20, 20, 10, 10, 10, 5, 5, 10, 20, 20, 5, 5, 5, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0,
    -10, -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub static KING_POSITIONAL_SCORE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 0, 0, 0, 5, 5, 10, 10, 5, 5, 0, 0, 5, 10, 20, 20, 10,
    5, 0, 0, 5, 10, 20, 20, 10, 5, 0, 0, 0, 5, 10, 10, 5, 0, 0, 0, 5, 5, -5, -5, 0, 5, 0, 0, 0, 5,
    0, -15, 0, 10, 0,
];

pub static MIRROR_SCORE: [Square; 64] = [
    Square::A1,
    Square::B1,
    Square::C1,
    Square::D1,
    Square::E1,
    Square::F1,
    Square::G1,
    Square::H1,
    Square::A2,
    Square::B2,
    Square::C2,
    Square::D2,
    Square::E2,
    Square::F2,
    Square::G2,
    Square::H2,
    Square::A3,
    Square::B3,
    Square::C3,
    Square::D3,
    Square::E3,
    Square::F3,
    Square::G3,
    Square::H3,
    Square::A4,
    Square::B4,
    Square::C4,
    Square::D4,
    Square::E4,
    Square::F4,
    Square::G4,
    Square::H4,
    Square::A5,
    Square::B5,
    Square::C5,
    Square::D5,
    Square::E5,
    Square::F5,
    Square::G5,
    Square::H5,
    Square::A6,
    Square::B6,
    Square::C6,
    Square::D6,
    Square::E6,
    Square::F6,
    Square::G6,
    Square::H6,
    Square::A7,
    Square::B7,
    Square::C7,
    Square::D7,
    Square::E7,
    Square::F7,
    Square::G7,
    Square::H7,
    Square::A8,
    Square::B8,
    Square::C8,
    Square::D8,
    Square::E8,
    Square::F8,
    Square::G8,
    Square::H8,
];
