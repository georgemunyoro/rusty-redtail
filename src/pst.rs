use crate::chess;

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

pub static MIRROR_SCORE: [chess::Square; 64] = [
    chess::Square::A1,
    chess::Square::B1,
    chess::Square::C1,
    chess::Square::D1,
    chess::Square::E1,
    chess::Square::F1,
    chess::Square::G1,
    chess::Square::H1,
    chess::Square::A2,
    chess::Square::B2,
    chess::Square::C2,
    chess::Square::D2,
    chess::Square::E2,
    chess::Square::F2,
    chess::Square::G2,
    chess::Square::H2,
    chess::Square::A3,
    chess::Square::B3,
    chess::Square::C3,
    chess::Square::D3,
    chess::Square::E3,
    chess::Square::F3,
    chess::Square::G3,
    chess::Square::H3,
    chess::Square::A4,
    chess::Square::B4,
    chess::Square::C4,
    chess::Square::D4,
    chess::Square::E4,
    chess::Square::F4,
    chess::Square::G4,
    chess::Square::H4,
    chess::Square::A5,
    chess::Square::B5,
    chess::Square::C5,
    chess::Square::D5,
    chess::Square::E5,
    chess::Square::F5,
    chess::Square::G5,
    chess::Square::H5,
    chess::Square::A6,
    chess::Square::B6,
    chess::Square::C6,
    chess::Square::D6,
    chess::Square::E6,
    chess::Square::F6,
    chess::Square::G6,
    chess::Square::H6,
    chess::Square::A7,
    chess::Square::B7,
    chess::Square::C7,
    chess::Square::D7,
    chess::Square::E7,
    chess::Square::F7,
    chess::Square::G7,
    chess::Square::H7,
    chess::Square::A8,
    chess::Square::B8,
    chess::Square::C8,
    chess::Square::D8,
    chess::Square::E8,
    chess::Square::F8,
    chess::Square::G8,
    chess::Square::H8,
];
