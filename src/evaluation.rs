use std::collections::HashMap;

use crate::{
    board::{Board, Position},
    chess,
    movegen::MoveGenerator,
    utils,
};

static ROOK_POSITIONAL_SCORE: [i32; 64] = [
    50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 0, 0, 10, 20, 20, 10, 0, 0, 0,
    0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20,
    20, 10, 0, 0, 0, 0, 0, 20, 20, 0, 0, 0,
];

static KNIGHT_POSITIONAL_SCORE: [i32; 64] = [
    -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 10, 10, 0, 0, -5, -5, 5, 20, 20, 20, 20, 5, -5, -5, 10, 20,
    30, 30, 20, 10, -5, -5, 10, 20, 30, 30, 20, 10, -5, -5, 5, 20, 10, 10, 20, 5, -5, -5, 0, 0, 0,
    0, 0, 0, -5, -5, -10, 0, 0, 0, 0, -10, -5,
];

static BISHOP_POSITIONAL_SCORE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 0, 0, 0, 0, 0, 10, 20, 20, 10,
    0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 10, 0, 0, 0, 0, 10, 0, 0, 30, 0, 0, 0, 0, 30, 0, 0, 0,
    -10, 0, 0, -10, 0, 0,
];

static PAWN_POSITIONAL_SCORE: [i32; 64] = [
    90, 90, 90, 90, 90, 90, 90, 90, 30, 30, 30, 40, 40, 30, 30, 30, 20, 20, 20, 30, 30, 30, 20, 20,
    10, 10, 10, 20, 20, 10, 10, 10, 5, 5, 10, 20, 20, 5, 5, 5, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0,
    -10, -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

static KING_POSITIONAL_SCORE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 0, 0, 0, 5, 5, 10, 10, 5, 5, 0, 0, 5, 10, 20, 20, 10,
    5, 0, 0, 5, 10, 20, 20, 10, 5, 0, 0, 0, 5, 10, 10, 5, 0, 0, 0, 5, 5, -5, -5, 0, 5, 0, 0, 0, 5,
    0, -15, 0, 10, 0,
];

static MIRROR_SCORE: [chess::Square; 64] = [
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

#[derive(Debug)]
pub struct PositionEvaluation {
    pub score: i32,
    pub best_move: Option<chess::Move>,
    pub depth: i32,
    pub ply: i32,
    pub nodes: i32,
}

pub struct Evaluator {
    pub running: bool,
    pub transposition_table: HashMap<u64, PositionEvaluation>,
    pub result: PositionEvaluation,
}

impl Evaluator {
    pub fn get_best_move(&mut self, position: &mut Position, depth: i32) -> Option<chess::Move> {
        self.result = PositionEvaluation {
            score: -50000,
            best_move: None,
            depth: 0,
            ply: 0,
            nodes: 0,
        };
        self.negamax(position, -50000, 50000, depth);
        return self.result.best_move;
    }

    pub fn negamax(&mut self, position: &mut Position, alpha: i32, beta: i32, depth: i32) -> i32 {
        if depth == 0 {
            return self.evaluate(position);
        }

        self.result.nodes += 1;

        let mut mut_alpha = alpha.clone();
        let mut best_move_so_far: Option<chess::Move> = None;

        let moves = position.generate_moves();

        for m in moves {
            self.result.ply += 1;

            let is_valid = position.make_move(m, false);
            if !is_valid {
                self.result.ply -= 1;
                continue;
            }

            let score = -self.negamax(position, -beta, -mut_alpha, depth - 1);

            position.unmake_move();
            self.result.ply -= 1;

            if score >= beta {
                return beta;
            }

            if score > alpha {
                mut_alpha = score;
                if self.result.ply == 0 {
                    best_move_so_far = Some(m);
                }
                self.result.depth = depth;
            }
        }

        if mut_alpha != alpha {
            self.result.best_move = best_move_so_far;
        }

        return mut_alpha;
    }

    pub fn get_piece_value(&mut self, piece: chess::Piece, square: usize) -> i32 {
        match piece {
            chess::Piece::WhitePawn => {
                let mut score = 100;
                score += PAWN_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteKnight => {
                let mut score = 300;
                score += KNIGHT_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteBishop => {
                let mut score = 350;
                score += BISHOP_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteRook => {
                let mut score = 500;
                score += ROOK_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteQueen => {
                let mut score = 1000;
                score += ROOK_POSITIONAL_SCORE[square] + BISHOP_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteKing => {
                let mut score = 10000;
                score += KING_POSITIONAL_SCORE[square];
                return score;
            }

            chess::Piece::BlackPawn => {
                let mut score = -100;
                score -= PAWN_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackKnight => {
                let mut score = -300;
                score -= KNIGHT_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackBishop => {
                let mut score = -350;
                score -= BISHOP_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackRook => {
                let mut score = -500;
                score -= ROOK_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackQueen => {
                let mut score = -1000;
                score -= ROOK_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize]
                    + BISHOP_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackKing => {
                let mut score = -10000;
                score -= KING_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }

            chess::Piece::Empty => 0,
        }
    }

    pub fn evaluate(&mut self, position: &mut Position) -> i32 {
        let mut score = 0;

        for piece in (chess::Piece::BlackPawn as usize)..=(chess::Piece::WhiteKing as usize) {
            let piece = chess::Piece::from(piece);
            let mut bitboard = position.bitboards[piece as usize];

            while bitboard != 0 {
                let square = utils::pop_lsb(&mut bitboard);
                score += self.get_piece_value(piece, square as usize);
            }
        }

        return if position.turn == chess::Color::White {
            score
        } else {
            -score
        };
    }
}
