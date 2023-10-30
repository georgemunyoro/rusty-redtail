use std::cmp::Ordering;
use std::{collections::HashMap, fmt::Display};

use crate::board::attacks::{KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS};
use crate::board::constants::squares_between;
use crate::chess::constants::FULL_BOARD;
use crate::{
    board::{
        constants::{
            BLACK_KING_SIDE_CASTLE, BLACK_QUEEN_SIDE_CASTLE, WHITE_KING_SIDE_CASTLE,
            WHITE_QUEEN_SIDE_CASTLE,
        },
        Board, Position,
    },
    chess::{
        self,
        castling_rights::CastlingRights,
        color::Color,
        constants::{RANK_2, RANK_4, RANK_5, RANK_6, RANK_7},
        piece::{Piece, PieceTrait},
        square::{Square, SQUARE_ITER},
    },
    utils::{self, _print_bitboard, clear_bit, get_bit},
};

pub trait MoveGenerator {
    fn generate_legal_moves(&mut self);
    fn generate_moves(&mut self, only_captures: bool) -> Vec<chess::_move::BitPackedMove>;

    fn generate_knight_moves(
        &self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    );
    fn generate_bishop_moves(
        &self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    );
    fn generate_rook_moves(
        &self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    );
    fn generate_queen_moves(
        &self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    );
    fn generate_king_moves(
        &mut self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    );
    fn generate_castle_moves(&self, move_list: &mut Vec<chess::_move::BitPackedMove>);
    fn generate_pawn_moves(
        &self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    );
    fn generate_white_pawn_moves(
        &self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    );
    fn generate_black_pawn_moves(
        &self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    );

    fn get_attacked_squares(&mut self) -> u64;
    fn get_square_attackers(&self, square: Square, color: Color) -> u64;
    // fn append_bb_movelist(&mut self, source_move_list: u64, piece: Piece, source: Square);
    fn get_between_squares(&self, source: Square, target: Square) -> u64;

    fn perft(&mut self, depth: u8) -> u64;
    fn divide_perft(&mut self, depth: u8) -> u64;
}

#[derive(Debug, Clone, Copy)]
pub struct PerftResult {
    pub depth: u8,
    pub nodes: u64,
    pub captures: u64,
    pub enpassants: u64,
    pub castles: u64,
    pub promotions: u64,
    pub checks: u64,
    pub discovery_checks: u64,
    pub double_checks: u64,
    pub checkmates: u64,
}

impl Display for PerftResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        result.push_str(format!("depth: {}\n", self.depth).as_str());
        result.push_str(format!("nodes: {}\n", self.nodes).as_str());
        result.push_str(format!("captures: {}\n", self.captures).as_str());
        result.push_str(format!("en_passant: {}\n", self.enpassants).as_str());
        result.push_str(format!("castles: {}\n", self.castles).as_str());
        result.push_str(format!("promotions: {}\n", self.promotions).as_str());
        result.push_str(format!("checks: {}\n", self.checks).as_str());
        result.push_str(format!("discovery_checks: {}\n", self.discovery_checks).as_str());
        result.push_str(format!("double_checks: {}\n", self.double_checks).as_str());
        result.push_str(format!("checkmates: {}\n", self.checkmates).as_str());

        write!(f, "{}", result)
    }
}

// implement addition of two PerftResult structs
impl std::ops::Add for PerftResult {
    type Output = PerftResult;

    fn add(self, other: PerftResult) -> PerftResult {
        PerftResult {
            depth: self.depth,
            nodes: self.nodes + other.nodes,
            captures: self.captures + other.captures,
            enpassants: self.enpassants + other.enpassants,
            castles: self.castles + other.castles,
            promotions: self.promotions + other.promotions,
            checks: self.checks + other.checks,
            discovery_checks: self.discovery_checks + other.discovery_checks,
            double_checks: self.double_checks + other.double_checks,
            checkmates: self.checkmates + other.checkmates,
        }
    }
}

static WHITE_PROMOTIONS: [Piece; 4] = [
    Piece::WhiteQueen,
    Piece::WhiteRook,
    Piece::WhiteBishop,
    Piece::WhiteKnight,
];

static BLACK_PROMOTIONS: [Piece; 4] = [
    Piece::BlackQueen,
    Piece::BlackRook,
    Piece::BlackBishop,
    Piece::BlackKnight,
];

impl MoveGenerator for Position {
    fn generate_legal_moves(&mut self) {
        self.move_list_stack[self.depth].clear();

        let enemy_pieces = self.occupancies[!self.turn as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let opponent_color = !self.turn;

        let (push_mask, capture_mask): (u64, u64);
        let (is_in_check, is_in_double_check): (bool, bool);

        let king_square = Square::from(utils::get_lsb(
            self.bitboards[Piece::WhiteKing as usize + (self.turn as usize * 6)],
        ));

        // ======================================================================
        // ============================= King moves =============================
        // ======================================================================
        let checkers = self.get_square_attackers(king_square, opponent_color);
        (is_in_check, is_in_double_check, capture_mask, push_mask) =
            self.compute_check_masks(king_square, checkers);
        let attacked_squares_wo_king = self.get_attacked_squares_without_king(king_square);

        let king_moves =
            KING_ATTACKS[king_square as usize] & !friendly_pieces & !attacked_squares_wo_king;

        self.append_bb_movelist_captures(
            king_moves & enemy_pieces,
            Piece::from(Piece::WhiteKing as usize + (self.turn as usize * 6)),
            king_square,
        );

        self.append_bb_movelist(
            king_moves & !enemy_pieces,
            Piece::from(Piece::WhiteKing as usize + (self.turn as usize * 6)),
            king_square,
        );

        if is_in_double_check {
            return;
        }

        // ======================================================================
        // =========================== Castling moves ===========================
        // ======================================================================
        if !is_in_check {
            if self.turn == Color::White {
                let is_king_side_empty = self.occupancies[2] & WHITE_KING_SIDE_CASTLE == 0;

                if is_king_side_empty && self.castling.can_castle(CastlingRights::WHITE_KINGSIDE) {
                    if !self.is_square_attacked(Square::E1, Color::Black)
                        && !self.is_square_attacked(Square::F1, Color::Black)
                        && !self.is_square_attacked(Square::G1, Color::Black)
                        && get_bit(self.bitboards[Piece::WhiteRook as usize], Square::H1 as u8) != 0
                    {
                        let mut m = chess::_move::BitPackedMove::new(
                            Square::E1,
                            Square::G1,
                            Piece::WhiteKing,
                        );
                        m.set_castle();
                        self.move_list_stack[self.depth].push(m);
                    }
                }

                let is_queen_side_empty = self.occupancies[2] & WHITE_QUEEN_SIDE_CASTLE == 0;

                if is_queen_side_empty && self.castling.can_castle(CastlingRights::WHITE_QUEENSIDE)
                {
                    if !self.is_square_attacked(Square::E1, Color::Black)
                        && !self.is_square_attacked(Square::D1, Color::Black)
                        && !self.is_square_attacked(Square::C1, Color::Black)
                        && get_bit(self.bitboards[Piece::WhiteRook as usize], Square::A1 as u8) != 0
                    {
                        let mut m = chess::_move::BitPackedMove::new(
                            Square::E1,
                            Square::C1,
                            Piece::WhiteKing,
                        );
                        m.set_castle();
                        self.move_list_stack[self.depth].push(m);
                    }
                }
            } else if self.turn == Color::Black {
                let is_king_side_empty = (self.occupancies[2] & BLACK_KING_SIDE_CASTLE) == 0;

                if is_king_side_empty && self.castling.can_castle(CastlingRights::BLACK_KINGSIDE) {
                    if !self.is_square_attacked(Square::E8, Color::White)
                        && !self.is_square_attacked(Square::F8, Color::White)
                        && !self.is_square_attacked(Square::G8, Color::White)
                        && get_bit(self.bitboards[Piece::BlackRook as usize], Square::H8 as u8) != 0
                    {
                        let mut m = chess::_move::BitPackedMove::new(
                            Square::E8,
                            Square::G8,
                            Piece::BlackKing,
                        );
                        m.set_castle();
                        self.move_list_stack[self.depth].push(m);
                    }
                }

                let is_queen_side_empty = (self.occupancies[2] & BLACK_QUEEN_SIDE_CASTLE) == 0;

                if is_queen_side_empty && self.castling.can_castle(CastlingRights::BLACK_QUEENSIDE)
                {
                    if !self.is_square_attacked(Square::E8, Color::White)
                        && !self.is_square_attacked(Square::D8, Color::White)
                        && !self.is_square_attacked(Square::C8, Color::White)
                        && get_bit(self.bitboards[Piece::BlackRook as usize], Square::A8 as u8) != 0
                    {
                        let mut m = chess::_move::BitPackedMove::new(
                            Square::E8,
                            Square::C8,
                            Piece::BlackKing,
                        );
                        m.set_castle();
                        self.move_list_stack[self.depth].push(m);
                    }
                }
            }
        }

        // ======================================================================
        // ========================= Pinned piece setup =========================
        // ======================================================================

        let mut pinned_pieces: u64 = 0;
        let mut pinned_piece_moves: HashMap<u64, u64> = HashMap::new();

        let opponent_sliders_mask = self.bitboards
            [Piece::WhiteQueen as usize + (opponent_color as usize * 6)]
            | self.bitboards[Piece::WhiteRook as usize + (opponent_color as usize * 6)]
            | self.bitboards[Piece::WhiteBishop as usize + (opponent_color as usize * 6)];

        let mut possible_pinner_squares = self.get_queen_magic_attacks(king_square, 0)
            & !KING_ATTACKS[king_square as usize] // Sliding pieces can't be pinners if they are next to the king.
            & opponent_sliders_mask;

        // For each of the possible pinners, check if there is a single friendly piece between
        // the sliding piece and the king. If so, then the friendly piece is pinned to the king.
        while possible_pinner_squares != 0 {
            let pinner_square = Square::from(utils::pop_lsb(&mut possible_pinner_squares));
            let pinner_piece = self.get_piece_at_square(pinner_square as u8);

            // First check if it attacks the king when all the friendly pieces are removed.

            let is_potential_rook_pinner = (pinner_piece.is_rook() || pinner_piece.is_queen())
                && ((self.get_rook_magic_attacks(
                    pinner_square,
                    enemy_pieces | (1u64 << king_square as u8),
                ) & self.bitboards[Piece::WhiteKing as usize + (self.turn as usize * 6)])
                    != 0);

            let is_potential_bishop_pinner = (pinner_piece.is_bishop() || pinner_piece.is_queen())
                && ((self.get_bishop_magic_attacks(
                    pinner_square,
                    enemy_pieces | (1u64 << king_square as u8),
                ) & self.bitboards[Piece::WhiteKing as usize + (self.turn as usize * 6)])
                    != 0);

            if !is_potential_bishop_pinner && !is_potential_rook_pinner {
                continue;
            }

            // If it does, then check if there is a single friendly piece between the
            // sliding piece and the king.

            let pieces_between_slider_and_king =
                self.get_between_squares(king_square, pinner_square);
            let friendly_pieces_between_slider_and_king =
                pieces_between_slider_and_king & friendly_pieces;

            if friendly_pieces_between_slider_and_king.count_ones() == 1 {
                pinned_piece_moves.insert(
                    friendly_pieces_between_slider_and_king,
                    pieces_between_slider_and_king | (1u64 << pinner_square as u8),
                );
                pinned_pieces |= friendly_pieces_between_slider_and_king;
            }
        }

        // ======================================================================
        // ============================== Knight moves ==========================
        // ======================================================================

        let mut movable_knights =
            self.bitboards[Piece::WhiteKnight as usize + (self.turn as usize * 6)] & !pinned_pieces;

        while movable_knights != 0 {
            let source = Square::from(utils::pop_lsb(&mut movable_knights));
            let knight_moves =
                KNIGHT_ATTACKS[source as usize] & !friendly_pieces & (push_mask | capture_mask);

            self.append_bb_movelist_captures(
                knight_moves & enemy_pieces,
                Piece::from(Piece::WhiteKnight as usize + (self.turn as usize * 6)),
                source,
            );

            self.append_bb_movelist(
                knight_moves & !enemy_pieces,
                Piece::from(Piece::WhiteKnight as usize + (self.turn as usize * 6)),
                source,
            );
        }

        // ======================================================================
        // ============================== Bishop moves ==========================
        // ======================================================================
        let mut bishops = self.bitboards[Piece::WhiteBishop as usize + (self.turn as usize * 6)];

        while bishops != 0 {
            let source = Square::from(utils::pop_lsb(&mut bishops));
            let mut bishop_moves = self.get_bishop_magic_attacks(source, self.occupancies[2])
                & !friendly_pieces
                & (push_mask | capture_mask);

            if pinned_pieces & (1u64 << source as u8) != 0 {
                bishop_moves &= pinned_piece_moves[&(1u64 << source as u8)];
            }

            self.append_bb_movelist_captures(
                bishop_moves & enemy_pieces,
                Piece::from(Piece::WhiteBishop as usize + (self.turn as usize * 6)),
                source,
            );

            self.append_bb_movelist(
                bishop_moves & !enemy_pieces,
                Piece::from(Piece::WhiteBishop as usize + (self.turn as usize * 6)),
                source,
            );
        }

        // ======================================================================
        // ============================== Rook moves ============================
        // ======================================================================
        let mut rooks = self.bitboards[Piece::WhiteRook as usize + (self.turn as usize * 6)];

        while rooks != 0 {
            let source = Square::from(utils::pop_lsb(&mut rooks));
            let mut rook_moves = self.get_rook_magic_attacks(source, self.occupancies[2])
                & !friendly_pieces
                & (push_mask | capture_mask);

            if pinned_pieces & (1u64 << source as u8) != 0 {
                rook_moves &= pinned_piece_moves[&(1u64 << source as u8)];
            }

            self.append_bb_movelist_captures(
                rook_moves & enemy_pieces,
                Piece::from(Piece::WhiteRook as usize + (self.turn as usize * 6)),
                source,
            );

            self.append_bb_movelist(
                rook_moves & !enemy_pieces,
                Piece::from(Piece::WhiteRook as usize + (self.turn as usize * 6)),
                source,
            );
        }

        // ======================================================================
        // ============================== Queen moves ===========================
        // ======================================================================
        let mut queens = self.bitboards[Piece::WhiteQueen as usize + (self.turn as usize * 6)];

        while queens != 0 {
            let source = Square::from(utils::pop_lsb(&mut queens));
            let mut queen_moves = self.get_queen_magic_attacks(source, self.occupancies[2])
                & !friendly_pieces
                & (push_mask | capture_mask);

            if pinned_pieces & (1u64 << source as u8) != 0 {
                queen_moves &= pinned_piece_moves[&(1u64 << source as u8)];
            }

            self.append_bb_movelist_captures(
                queen_moves & enemy_pieces,
                Piece::from(Piece::WhiteQueen as usize + (self.turn as usize * 6)),
                source,
            );

            self.append_bb_movelist(
                queen_moves & !enemy_pieces,
                Piece::from(Piece::WhiteQueen as usize + (self.turn as usize * 6)),
                source,
            );
        }

        // ======================================================================
        // ============================== Pawn moves ============================
        // ======================================================================
        let mut pawns = self.bitboards[Piece::WhitePawn as usize + (self.turn as usize * 6)];

        while pawns != 0 {
            let source = Square::from(utils::pop_lsb(&mut pawns));

            // Enpassant captures
            if let Some(enpassant_square) = self.enpassant {
                let enpassant_attacks = PAWN_ATTACKS[self.turn as usize][source as usize]
                    & (1u64 << enpassant_square as u8);

                if enpassant_attacks != 0 {
                    let target_enpassant = utils::get_lsb(enpassant_attacks);
                    let mut m = chess::_move::BitPackedMove::new(
                        source,
                        Square::from(target_enpassant),
                        Piece::from(Piece::WhitePawn as usize + (self.turn as usize * 6)),
                    );
                    m.set_enpassant();

                    debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
                    unsafe {
                        self.move_list_stack.get_unchecked_mut(self.depth).push(m);
                    }
                }
            }

            // Captures
            let mut pawn_captures = PAWN_ATTACKS[self.turn as usize][source as usize]
                & enemy_pieces
                & (push_mask | capture_mask);

            if pinned_pieces & (1u64 << source as u8) != 0 {
                pawn_captures &= pinned_piece_moves[&(1u64 << source as u8)];
            }

            let pawn_piece: Piece;
            let mut pawn_pushes: u64 = 0;

            // Pushes
            if self.turn == Color::White {
                pawn_piece = Piece::WhitePawn;

                let single_push =
                    (1u64 << (source as u8 - 8)) & !(enemy_pieces | friendly_pieces) & (push_mask);

                let double_push =
                    (((1u64 << (source as u8 - 8)) & !(enemy_pieces | friendly_pieces)) >> 8)
                        & (push_mask)
                        & !(enemy_pieces | friendly_pieces)
                        & RANK_4;

                pawn_pushes = single_push | double_push;

                if pinned_pieces & (1u64 << source as u8) != 0 {
                    pawn_pushes &= pinned_piece_moves[&(1u64 << source as u8)];
                }
            } else {
                pawn_piece = Piece::BlackPawn;

                let single_push =
                    (1u64 << (source as u8 + 8)) & !(enemy_pieces | friendly_pieces) & (push_mask);

                let double_push =
                    (((1u64 << (source as u8 + 8)) & !(enemy_pieces | friendly_pieces)) << 8)
                        & (push_mask)
                        & !(enemy_pieces | friendly_pieces)
                        & RANK_5;

                pawn_pushes = single_push | double_push;

                if pinned_pieces & (1u64 << source as u8) != 0 {
                    pawn_pushes &= pinned_piece_moves[&(1u64 << source as u8)];
                }
            }

            while pawn_captures != 0 {
                let target = Square::from(utils::pop_lsb(&mut pawn_captures));
                let mut m = chess::_move::BitPackedMove::new(source, target, pawn_piece);

                m.set_capture(self.get_piece_at_square(target as u8));

                match pawn_piece {
                    Piece::WhitePawn if target <= Square::H8 => {
                        for &promotion in &WHITE_PROMOTIONS {
                            let mut promotion_move = m;
                            promotion_move.set_promotion(promotion);

                            debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
                            unsafe {
                                self.move_list_stack
                                    .get_unchecked_mut(self.depth)
                                    .push(promotion_move);
                            }
                        }
                    }
                    Piece::BlackPawn if target >= Square::A1 => {
                        for &promotion in &BLACK_PROMOTIONS {
                            let mut promotion_move = m;
                            promotion_move.set_promotion(promotion);

                            debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
                            unsafe {
                                self.move_list_stack
                                    .get_unchecked_mut(self.depth)
                                    .push(promotion_move);
                            }
                        }
                    }
                    _ => {
                        debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
                        unsafe {
                            self.move_list_stack.get_unchecked_mut(self.depth).push(m);
                        }
                    }
                }
            }

            while pawn_pushes != 0 {
                let target = Square::from(utils::pop_lsb(&mut pawn_pushes));
                let m = chess::_move::BitPackedMove::new(source, target, pawn_piece);

                match pawn_piece {
                    Piece::WhitePawn if target <= Square::H8 => {
                        for &promotion in &WHITE_PROMOTIONS {
                            let mut promotion_move = m;
                            promotion_move.set_promotion(promotion);

                            debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
                            unsafe {
                                self.move_list_stack
                                    .get_unchecked_mut(self.depth)
                                    .push(promotion_move);
                            }
                        }
                    }
                    Piece::BlackPawn if target >= Square::A1 => {
                        for &promotion in &BLACK_PROMOTIONS {
                            let mut promotion_move = m;
                            promotion_move.set_promotion(promotion);

                            debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
                            unsafe {
                                self.move_list_stack
                                    .get_unchecked_mut(self.depth)
                                    .push(promotion_move);
                            }
                        }
                    }
                    _ => {
                        debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
                        unsafe {
                            self.move_list_stack.get_unchecked_mut(self.depth).push(m);
                        }
                    }
                }
            }
        }
    }

    fn generate_moves(&mut self, only_captures: bool) -> Vec<chess::_move::BitPackedMove> {
        let mut moves = Vec::with_capacity(256);

        self.generate_pawn_moves(&mut moves, only_captures);
        self.generate_knight_moves(&mut moves, only_captures);
        self.generate_bishop_moves(&mut moves, only_captures);
        self.generate_rook_moves(&mut moves, only_captures);
        self.generate_queen_moves(&mut moves, only_captures);
        self.generate_king_moves(&mut moves, only_captures);

        return moves;
    }

    fn generate_knight_moves(
        &self,
        moves: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    ) {
        let piece = if self.turn == Color::White {
            Piece::WhiteKnight
        } else {
            Piece::BlackKnight
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let enemy_pieces = self.occupancies[!self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut knight_moves = KNIGHT_ATTACKS[i as usize] & !friendly_pieces;
            if only_captures {
                knight_moves &= enemy_pieces
            }

            while knight_moves != 0 {
                let j = utils::pop_lsb(&mut knight_moves);
                let mut m =
                    chess::_move::BitPackedMove::new(Square::from(i), Square::from(j), piece);

                if get_bit(self.occupancies[2], j) != 0 {
                    m.set_capture(self.get_piece_at_square(j));
                }

                moves.push(m);
            }
        }
    }

    fn generate_bishop_moves(
        &self,
        moves: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    ) {
        let piece = if self.turn == Color::White {
            Piece::WhiteBishop
        } else {
            Piece::BlackBishop
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let enemy_pieces = self.occupancies[!self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut bishop_moves = self
                .get_bishop_magic_attacks(Square::from(i), friendly_pieces | enemy_pieces)
                & !friendly_pieces;

            if only_captures {
                bishop_moves &= enemy_pieces;
            }

            while bishop_moves != 0 {
                let j = utils::pop_lsb(&mut bishop_moves);
                let mut m =
                    chess::_move::BitPackedMove::new(Square::from(i), Square::from(j), piece);

                if get_bit(self.occupancies[2], j) != 0 {
                    m.set_capture(self.get_piece_at_square(j));
                }

                moves.push(m);
            }
        }
    }

    fn generate_rook_moves(
        &self,
        moves: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    ) {
        let piece = if self.turn == Color::White {
            Piece::WhiteRook
        } else {
            Piece::BlackRook
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let enemy_pieces = self.occupancies[!self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut rook_moves = self
                .get_rook_magic_attacks(Square::from(i), friendly_pieces | enemy_pieces)
                & !friendly_pieces;

            if only_captures {
                rook_moves &= enemy_pieces;
            }

            while rook_moves != 0 {
                let j = utils::pop_lsb(&mut rook_moves);
                let mut m =
                    chess::_move::BitPackedMove::new(Square::from(i), Square::from(j), piece);

                if get_bit(self.occupancies[2], j) != 0 {
                    let captured_piece = self.get_piece_at_square(j);
                    m.set_capture(captured_piece);
                }

                moves.push(m);
            }
        }
    }

    fn generate_queen_moves(
        &self,
        moves: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    ) {
        let piece = if self.turn == Color::White {
            Piece::WhiteQueen
        } else {
            Piece::BlackQueen
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let enemy_pieces = self.occupancies[!self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut queen_moves = self
                .get_queen_magic_attacks(Square::from(i), friendly_pieces | enemy_pieces)
                & !friendly_pieces;

            if only_captures {
                queen_moves &= enemy_pieces;
            }

            while queen_moves != 0 {
                let j = utils::pop_lsb(&mut queen_moves);
                let mut m =
                    chess::_move::BitPackedMove::new(Square::from(i), Square::from(j), piece);

                if get_bit(self.occupancies[2], j) != 0 {
                    m.set_capture(self.get_piece_at_square(j));
                }

                moves.push(m);
            }
        }
    }

    fn generate_king_moves(
        &mut self,
        moves: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    ) {
        let piece = if self.turn == Color::White {
            Piece::WhiteKing
        } else {
            Piece::BlackKing
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let enemy_pieces = self.occupancies[!self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut king_moves = KING_ATTACKS[i as usize] & !friendly_pieces;

            if only_captures {
                king_moves &= enemy_pieces;
            }

            while king_moves != 0 {
                let j = utils::pop_lsb(&mut king_moves);

                clear_bit(&mut self.bitboards[piece as usize], i);
                let is_target_square_attacked = self.is_square_attacked_w_occupancy(
                    Square::from(j),
                    !self.turn,
                    self.get_both_occupancy(),
                );
                utils::set_bit(&mut self.bitboards[piece as usize], i);

                if is_target_square_attacked {
                    continue;
                }

                let mut m =
                    chess::_move::BitPackedMove::new(Square::from(i), Square::from(j), piece);

                if get_bit(self.occupancies[2], j) != 0 {
                    m.set_capture(self.get_piece_at_square(j));
                }

                moves.push(m);
            }
        }

        if !only_captures {
            self.generate_castle_moves(moves);
        }
    }

    fn generate_castle_moves(&self, moves: &mut Vec<chess::_move::BitPackedMove>) {
        if self.is_in_check() {
            return;
        }

        if self.turn == Color::White {
            let is_king_side_empty = self.occupancies[2] & WHITE_KING_SIDE_CASTLE == 0;

            if is_king_side_empty && self.castling.can_castle(CastlingRights::WHITE_KINGSIDE) {
                if !self.is_square_attacked(Square::E1, Color::Black)
                    && !self.is_square_attacked(Square::F1, Color::Black)
                    && !self.is_square_attacked(Square::G1, Color::Black)
                    && get_bit(self.bitboards[Piece::WhiteRook as usize], Square::H1 as u8) != 0
                {
                    let mut m =
                        chess::_move::BitPackedMove::new(Square::E1, Square::G1, Piece::WhiteKing);
                    m.set_castle();
                    moves.push(m);
                }
            }

            let is_queen_side_empty = self.occupancies[2] & WHITE_QUEEN_SIDE_CASTLE == 0;

            if is_queen_side_empty && self.castling.can_castle(CastlingRights::WHITE_QUEENSIDE) {
                if !self.is_square_attacked(Square::E1, Color::Black)
                    && !self.is_square_attacked(Square::D1, Color::Black)
                    && !self.is_square_attacked(Square::C1, Color::Black)
                    && get_bit(self.bitboards[Piece::WhiteRook as usize], Square::A1 as u8) != 0
                {
                    let mut m =
                        chess::_move::BitPackedMove::new(Square::E1, Square::C1, Piece::WhiteKing);
                    m.set_castle();
                    moves.push(m);
                }
            }
        } else if self.turn == Color::Black {
            let is_king_side_empty = (self.occupancies[2] & BLACK_KING_SIDE_CASTLE) == 0;

            if is_king_side_empty && self.castling.can_castle(CastlingRights::BLACK_KINGSIDE) {
                if !self.is_square_attacked(Square::E8, Color::White)
                    && !self.is_square_attacked(Square::F8, Color::White)
                    && !self.is_square_attacked(Square::G8, Color::White)
                    && get_bit(self.bitboards[Piece::BlackRook as usize], Square::H8 as u8) != 0
                {
                    let mut m =
                        chess::_move::BitPackedMove::new(Square::E8, Square::G8, Piece::BlackKing);
                    m.set_castle();
                    moves.push(m);
                }
            }

            let is_queen_side_empty = (self.occupancies[2] & BLACK_QUEEN_SIDE_CASTLE) == 0;

            if is_queen_side_empty && self.castling.can_castle(CastlingRights::BLACK_QUEENSIDE) {
                if !self.is_square_attacked(Square::E8, Color::White)
                    && !self.is_square_attacked(Square::D8, Color::White)
                    && !self.is_square_attacked(Square::C8, Color::White)
                    && get_bit(self.bitboards[Piece::BlackRook as usize], Square::A8 as u8) != 0
                {
                    let mut m =
                        chess::_move::BitPackedMove::new(Square::E8, Square::C8, Piece::BlackKing);
                    m.set_castle();
                    moves.push(m);
                }
            }
        }
    }

    fn generate_pawn_moves(
        &self,
        move_list: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    ) {
        // white pawn moves
        if self.turn == Color::White {
            self.generate_white_pawn_moves(move_list, only_captures)
        }
        // black pawn moves
        else if self.turn == Color::Black {
            self.generate_black_pawn_moves(move_list, only_captures)
        }
    }

    fn generate_white_pawn_moves(
        &self,
        moves: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    ) {
        let mut piece_bitboard =
            self.bitboards[Piece::WhitePawn as usize] & !chess::constants::RANK_8;

        while piece_bitboard != 0 {
            let source = Square::from(utils::get_lsb(piece_bitboard));
            let target = Square::from((source as usize) - 8);

            // quiet pawn moves
            if !only_captures
                && (target >= Square::A8)
                && get_bit(self.occupancies[2], target as u8) == 0
            {
                // pawn promotion
                if get_bit(*RANK_7, source as u8) != 0 {
                    moves.extend(
                        [
                            Piece::WhiteQueen,
                            Piece::WhiteRook,
                            Piece::WhiteBishop,
                            Piece::WhiteKnight,
                        ]
                        .iter()
                        .map(|promotion| {
                            let mut m =
                                chess::_move::BitPackedMove::new(source, target, Piece::WhitePawn);
                            m.set_promotion(*promotion);
                            return m;
                        }),
                    );
                } else {
                    // single pawn push
                    moves.push(chess::_move::BitPackedMove::new(
                        source,
                        target,
                        Piece::WhitePawn,
                    ));

                    // double pawn push
                    if (source >= Square::A2) && (source <= Square::H2) {
                        let target = Square::from((source as u8) - 16);
                        if get_bit(self.occupancies[2], target as u8) == 0 {
                            moves.push(chess::_move::BitPackedMove::new(
                                source,
                                target,
                                Piece::WhitePawn,
                            ));
                        }
                    }
                }
            }

            // pawn captures
            let mut attacks = PAWN_ATTACKS[Color::White as usize][source as usize]
                & self.occupancies[Color::Black as usize];
            while attacks != 0 {
                let target = Square::from(utils::get_lsb(attacks));

                if get_bit(*RANK_7, source as u8) != 0 {
                    moves.extend(
                        [
                            Piece::WhiteQueen,
                            Piece::WhiteRook,
                            Piece::WhiteBishop,
                            Piece::WhiteKnight,
                        ]
                        .iter()
                        .map(|promotion| {
                            let mut m =
                                chess::_move::BitPackedMove::new(source, target, Piece::WhitePawn);
                            m.set_promotion(*promotion);
                            m.set_capture(self.get_piece_at_square(target as u8));
                            return m;
                        }),
                    );
                } else {
                    let mut m = chess::_move::BitPackedMove::new(source, target, Piece::WhitePawn);
                    m.set_capture(self.get_piece_at_square(target as u8));
                    moves.push(m);
                }

                utils::clear_bit(&mut attacks, target as u8);
            }

            // generate enpassant captures
            if let Some(enpassant_square) = self.enpassant {
                let enpassant_attacks = PAWN_ATTACKS[Color::White as usize][source as usize]
                    & (1u64 << enpassant_square as u8);

                if enpassant_attacks != 0 {
                    let target_enpassant = utils::get_lsb(enpassant_attacks);
                    let mut m = chess::_move::BitPackedMove::new(
                        source,
                        Square::from(target_enpassant),
                        Piece::WhitePawn,
                    );
                    m.set_enpassant();
                    moves.push(m);
                }
            }

            utils::pop_lsb(&mut piece_bitboard);
        }
    }

    fn generate_black_pawn_moves(
        &self,
        moves: &mut Vec<chess::_move::BitPackedMove>,
        only_captures: bool,
    ) {
        let mut piece_bitboard =
            self.bitboards[Piece::BlackPawn as usize] & !chess::constants::RANK_1;

        while piece_bitboard != 0 {
            let source = Square::from(utils::get_lsb(piece_bitboard));
            let target = Square::from((source as u8) + 8);

            // quiet pawn moves
            if !only_captures
                && (target <= Square::H1)
                && get_bit(self.occupancies[2], target as u8) == 0
            {
                // pawn promotion
                if get_bit(*RANK_2, source as u8) != 0 {
                    moves.extend(
                        [
                            Piece::BlackQueen,
                            Piece::BlackRook,
                            Piece::BlackBishop,
                            Piece::BlackKnight,
                        ]
                        .iter()
                        .map(|promotion| {
                            let mut m =
                                chess::_move::BitPackedMove::new(source, target, Piece::BlackPawn);
                            m.set_promotion(*promotion);
                            return m;
                        }),
                    );
                } else {
                    // single pawn push
                    moves.push(chess::_move::BitPackedMove::new(
                        source,
                        target,
                        Piece::BlackPawn,
                    ));

                    // double pawn push
                    if get_bit(*RANK_7, source as u8) != 0 {
                        let target = Square::from((source as u8) + 16);
                        if get_bit(self.occupancies[2], target as u8) == 0 {
                            moves.push(chess::_move::BitPackedMove::new(
                                source,
                                target,
                                Piece::BlackPawn,
                            ));
                        }
                    }
                }
            }

            // pawn captures
            let mut attacks = PAWN_ATTACKS[Color::Black as usize][source as usize]
                & self.occupancies[Color::White as usize];
            while attacks != 0 {
                let target = Square::from(utils::get_lsb(attacks));

                if source >= Square::A2 && source <= Square::H2 {
                    moves.extend(
                        [
                            Piece::BlackQueen,
                            Piece::BlackRook,
                            Piece::BlackBishop,
                            Piece::BlackKnight,
                        ]
                        .iter()
                        .map(|promotion| {
                            let mut m =
                                chess::_move::BitPackedMove::new(source, target, Piece::BlackPawn);
                            m.set_promotion(*promotion);
                            m.set_capture(self.get_piece_at_square(target as u8));
                            return m;
                        }),
                    );
                } else {
                    let mut m = chess::_move::BitPackedMove::new(source, target, Piece::BlackPawn);
                    m.set_capture(self.get_piece_at_square(target as u8));
                    moves.push(m);
                }

                utils::clear_bit(&mut attacks, target as u8);
            }

            // generate enpassant captures
            if let Some(enpassant_square) = self.enpassant {
                let enpassant_attacks = PAWN_ATTACKS[Color::Black as usize][source as usize]
                    & (1u64 << enpassant_square as u8);

                if enpassant_attacks != 0 {
                    let target_enpassant = utils::get_lsb(enpassant_attacks);
                    let mut m = chess::_move::BitPackedMove::new(
                        source,
                        Square::from(target_enpassant),
                        Piece::BlackPawn,
                    );
                    m.set_enpassant();
                    moves.push(m);
                }
            }

            utils::pop_lsb(&mut piece_bitboard);
        }
    }

    fn get_attacked_squares(&mut self) -> u64 {
        let mut attacked_squares: u64 = 0;
        for square in SQUARE_ITER {
            if self.is_square_attacked(square, !self.turn) {
                attacked_squares |= 1u64 << square as u8;
            }
        }
        return attacked_squares;
    }

    fn get_square_attackers(&self, square: Square, color: Color) -> u64 {
        let mut attackers: u64 = 0;

        // Pawn attackers
        attackers |= PAWN_ATTACKS[!color as usize][square as usize]
            & self.bitboards[Piece::WhitePawn as usize + (color as usize * 6)];

        // Knight attackers
        attackers |= KNIGHT_ATTACKS[square as usize]
            & self.bitboards[Piece::WhiteKnight as usize + (color as usize * 6)];

        // Diagonal slider attackers
        attackers |= self.get_bishop_magic_attacks(square, self.get_both_occupancy())
            & (self.bitboards[Piece::WhiteBishop as usize + (color as usize * 6)]
                | self.bitboards[Piece::WhiteQueen as usize + (color as usize * 6)]);

        // Orthogonal slider attackers
        attackers |= self.get_rook_magic_attacks(square, self.get_both_occupancy())
            & (self.bitboards[Piece::WhiteRook as usize + (color as usize * 6)]
                | self.bitboards[Piece::WhiteQueen as usize + (color as usize * 6)]);

        return attackers;
    }

    fn get_between_squares(&self, source: Square, target: Square) -> u64 {
        return squares_between(source as usize, target as usize);
    }

    fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut nodes = 0;

        self.generate_legal_moves();
        let num_moves = self.move_list_stack[self.depth].len();

        for i in 0..num_moves {
            let m = self.move_list_stack[self.depth][i];

            if !self.make_move(&m, false) {
                if m.is_enpassant() {
                    continue;
                }
                self.draw();
                panic!("Illegal move: {}", m);
            }

            nodes += self.perft(depth - 1);
            self.unmake_move();
        }

        return nodes;
    }

    fn divide_perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        self.generate_legal_moves();

        println!(
            "{}: {}: {}",
            depth,
            self.move_list_stack[self.depth].len(),
            self.depth
        );

        let mut nodes = 0;
        let num_moves = self.move_list_stack[self.depth].len();

        for i in 0..num_moves {
            let m = self.move_list_stack[self.depth][i];

            if !self.make_move(&m, false) {
                if m.is_enpassant() {
                    continue;
                }
                self.draw();
                panic!("Illegal move: {}", m);
            }

            let n = self.perft(depth - 1);
            nodes += n;
            self.unmake_move();

            println!("{}: {}", m, n);
        }

        return nodes;
    }
}

impl Position {
    fn get_pawn_attacks(&self) -> u64 {
        unsafe {
            let mut attacks: u64 = 0;
            let mut opponent_pawns = *self
                .bitboards
                .get_unchecked(Piece::WhitePawn as usize + (!self.turn as usize * 6));
            while opponent_pawns != 0 {
                let i = utils::pop_lsb(&mut opponent_pawns);
                attacks |= *PAWN_ATTACKS
                    .get_unchecked(!self.turn as usize)
                    .get_unchecked(i as usize);
            }
            return attacks;
        }
    }

    fn get_bishop_attacks(&self, occupancy: u64) -> u64 {
        let mut attacks: u64 = 0;
        let mut opponent_bishops =
            self.bitboards[Piece::WhiteBishop as usize + (!self.turn as usize * 6)];
        while opponent_bishops != 0 {
            let i = utils::pop_lsb(&mut opponent_bishops);
            attacks |= self.get_bishop_magic_attacks(Square::from(i), occupancy);
        }
        return attacks;
    }

    fn get_rook_attacks(&self, occupancy: u64) -> u64 {
        let mut attacks: u64 = 0;
        let mut opponent_rooks =
            self.bitboards[Piece::WhiteRook as usize + (!self.turn as usize * 6)];
        while opponent_rooks != 0 {
            let i = utils::pop_lsb(&mut opponent_rooks);
            attacks |= self.get_rook_magic_attacks(Square::from(i), occupancy);
        }
        return attacks;
    }

    fn get_queen_attacks(&self, occupancy: u64) -> u64 {
        let mut attacks: u64 = 0;
        let mut opponent_queens =
            self.bitboards[Piece::WhiteQueen as usize + (!self.turn as usize * 6)];
        while opponent_queens != 0 {
            let i = utils::pop_lsb(&mut opponent_queens);
            attacks |= self.get_queen_magic_attacks(Square::from(i), occupancy);
        }
        return attacks;
    }

    fn get_knight_attacks(&self) -> u64 {
        let mut attacks: u64 = 0;
        let mut opponent_knights =
            self.bitboards[Piece::WhiteKnight as usize + (!self.turn as usize * 6)];
        while opponent_knights != 0 {
            let i = utils::pop_lsb(&mut opponent_knights);
            attacks |= unsafe { KNIGHT_ATTACKS.get_unchecked(i as usize) };
        }
        return attacks;
    }

    fn get_attacked_squares_without_king(&mut self, king_square: Square) -> u64 {
        let occupancy_wo_king = self.get_both_occupancy() ^ (1u64 << king_square as u8);
        return self.get_bishop_attacks(occupancy_wo_king)
            | self.get_rook_attacks(occupancy_wo_king)
            | self.get_queen_attacks(occupancy_wo_king)
            | self.get_knight_attacks()
            | self.get_pawn_attacks();
    }

    fn compute_check_masks(
        &mut self,
        king_square: Square,
        checkers: u64,
    ) -> (bool, bool, u64, u64) {
        return match checkers.count_ones().cmp(&1) {
            Ordering::Equal => {
                let checker_square = Square::from(utils::get_lsb(checkers));
                let push_mask = if self.get_piece_at_square(checker_square as u8).is_slider() {
                    self.get_between_squares(king_square, checker_square)
                } else {
                    0
                };
                return (true, false, checkers, push_mask);
            }
            Ordering::Greater => (true, true, FULL_BOARD, FULL_BOARD),
            Ordering::Less => (false, false, FULL_BOARD, FULL_BOARD),
        };
    }

    fn append_bb_movelist_captures(&mut self, source_move_list: u64, piece: Piece, source: Square) {
        let mut move_list = source_move_list;
        let mut m = chess::_move::BitPackedMove::new(source, Square::A1, piece);

        while move_list != 0 {
            let target = utils::pop_lsb(&mut move_list);
            m.set_to(Square::from(target));
            m.set_capture(self.get_piece_at_square(target));

            debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
            unsafe {
                self.move_list_stack.get_unchecked_mut(self.depth).push(m);
            }
            m.set_capture(Piece::Empty);
        }
    }

    fn append_bb_movelist(&mut self, source_move_list: u64, piece: Piece, source: Square) {
        let mut move_list = source_move_list;
        let mut m = chess::_move::BitPackedMove::new(source, Square::A1, piece);

        while move_list != 0 {
            let target = utils::pop_lsb(&mut move_list);
            m.set_to(Square::from(target));

            debug_assert!(self.depth < 256, "Depth is too high: {}", self.depth);
            unsafe {
                self.move_list_stack.get_unchecked_mut(self.depth).push(m);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{self, Board};

    use super::MoveGenerator;

    #[test]
    fn get_piece_at_square_and_set_fen_works() {
        let mut position = board::Position::new(Some(crate::chess::constants::STARTING_FEN));
        assert!(position.perft(0) == 1);
        assert!(position.perft(1) == 20);
        assert!(position.perft(2) == 400);
        assert!(position.perft(3) == 8902);
        assert!(position.perft(4) == 197281);
    }
}
