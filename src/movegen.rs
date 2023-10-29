use std::{collections::HashMap, fmt::Display};

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
    fn generate_legal_moves(&mut self) -> Vec<chess::_move::BitPackedMove>;
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
    fn append_bb_movelist(
        &self,
        source_move_list: u64,
        target_move_list: &mut Vec<chess::_move::BitPackedMove>,
        piece: Piece,
        source: Square,
    );
    fn get_between_squares(&self, source: Square, target: Square) -> u64;

    fn perft(&mut self, depth: u8) -> u64;
    fn divide_perft(&mut self, depth: u8) -> u64;
    fn detailed_perft(&mut self, depth: u8) -> PerftResult;
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

impl MoveGenerator for Position {
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
            let mut attacks = self.pawn_attacks[Color::Black as usize][source as usize]
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
                let enpassant_attacks = self.pawn_attacks[Color::Black as usize][source as usize]
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
            let mut attacks = self.pawn_attacks[Color::White as usize][source as usize]
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
                let enpassant_attacks = self.pawn_attacks[Color::White as usize][source as usize]
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
            let mut knight_moves = self.knight_attacks[i as usize] & !friendly_pieces;
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
            let mut king_moves = self.king_attacks[i as usize] & !friendly_pieces;

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
        attackers |= self.pawn_attacks[!color as usize][square as usize]
            & self.bitboards[Piece::WhitePawn as usize + (color as usize * 6)];

        // Knight attackers
        attackers |= self.knight_attacks[square as usize]
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

    fn append_bb_movelist(
        &self,
        source_move_list: u64,
        target_move_list: &mut Vec<chess::_move::BitPackedMove>,
        piece: Piece,
        source: Square,
    ) {
        let mut move_list = source_move_list;
        while move_list != 0 {
            let target = Square::from(utils::pop_lsb(&mut move_list));
            let mut m = chess::_move::BitPackedMove::new(source, target, piece);
            if get_bit(self.occupancies[2], target as u8) != 0 {
                m.set_capture(self.get_piece_at_square(target as u8));
            }

            if (piece == Piece::WhitePawn && target <= Square::H7)
                || (piece == Piece::BlackPawn && target >= Square::A2)
            {
                for promotion in if piece.is_white() {
                    [
                        Piece::WhiteQueen,
                        Piece::WhiteRook,
                        Piece::WhiteBishop,
                        Piece::WhiteKnight,
                    ]
                } else {
                    [
                        Piece::BlackQueen,
                        Piece::BlackRook,
                        Piece::BlackBishop,
                        Piece::BlackKnight,
                    ]
                }
                .iter()
                {
                    let mut m = m;
                    m.set_promotion(*promotion);
                    target_move_list.push(m);
                }
            } else {
                target_move_list.push(m);
            }
        }
    }

    fn get_between_squares(&self, source: Square, target: Square) -> u64 {
        let mut between_squares: u64 = 0;

        let dx = (target as i8 % 8) - (source as i8 % 8);
        let dy = (target as i8 / 8) - (source as i8 / 8);

        let mut x = source as i8 % 8;
        let mut y = source as i8 / 8;

        let x_to_add = if dx > 0 {
            1
        } else {
            if dx < 0 {
                -1
            } else {
                0
            }
        };
        let y_to_add = if dy > 0 {
            1
        } else {
            if dy < 0 {
                -1
            } else {
                0
            }
        };

        x += x_to_add;
        y += y_to_add;

        // println!(
        //     "dx: {}, dy: {}, x: {}, y: {}, x_to_add: {}, y_to_add: {}",
        //     dx, dy, x, y, x_to_add, y_to_add
        // );

        while x != (target as i8 % 8) || y != (target as i8 / 8) {
            utils::set_bit(&mut between_squares, (x + y * 8) as u8);

            x += x_to_add;
            y += y_to_add;
        }

        return between_squares;
    }

    fn generate_legal_moves(&mut self) -> Vec<chess::_move::BitPackedMove> {
        let mut moves = Vec::with_capacity(256);

        let enemy_pieces = self.occupancies[!self.turn as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];

        let attacked_squares: u64 = self.get_attacked_squares();

        // println!("\nAttacked squares:");
        // _print_bitboard(attacked_squares);

        let mut push_mask: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let mut capture_mask: u64 = 0xFFFF_FFFF_FFFF_FFFF;

        let is_in_check: bool;
        let is_in_double_check: bool;

        let king_square = Square::from(utils::get_lsb(
            self.bitboards[Piece::WhiteKing as usize + (self.turn as usize * 6)],
        ));

        // ======================================================================
        // ============================= King moves =============================
        // ======================================================================
        {
            // Get all the "danger squares", i.e. squares that aren't attacked by the enemy
            // presently, but will be attacked if the king moves to that square. e.g. the
            // squares behind a king attacked by a rook, bishop, or queen.
            let mut attacked_squares_wo_king: u64 = 0;
            let occupancy_wo_king = self.get_both_occupancy() ^ (1u64 << king_square as u8);
            for square in SQUARE_ITER {
                if self.is_square_attacked_w_occupancy(square, !self.turn, occupancy_wo_king) {
                    attacked_squares_wo_king |= 1u64 << square as u8;
                }
            }

            let checkers = self.get_square_attackers(king_square, !self.turn);
            is_in_check = checkers != 0;
            is_in_double_check = checkers.count_ones() > 1;

            if is_in_check && !is_in_double_check {
                capture_mask = checkers;

                let checker_square = Square::from(utils::get_lsb(checkers));

                if self.get_piece_at_square(checker_square as u8).is_slider() {
                    // println!("\nBetween squares:");
                    // _print_bitboard(self.get_between_squares(king_square, checker_square));
                    push_mask = self.get_between_squares(king_square, checker_square);
                } else {
                    push_mask = 0;
                }
            }

            // TODO: enpassant

            // println!("\nCheckers:");
            // _print_bitboard(checkers);

            // println!("\nDanger squares:");
            // _print_bitboard(attacked_squares_wo_king);

            let king_moves = self.king_attacks[king_square as usize]
                & !friendly_pieces
                & !attacked_squares_wo_king;

            // println!("\nKing moves:");
            // _print_bitboard(king_moves);

            self.append_bb_movelist(
                king_moves,
                &mut moves,
                Piece::from(Piece::WhiteKing as usize + (self.turn as usize * 6)),
                king_square,
            );

            if is_in_double_check {
                return moves;
            }
        }

        // ======================================================================
        // =========================== Castling moves ===========================
        // ======================================================================
        {
            if !is_in_check {
                self.generate_castle_moves(&mut moves);
            }
        }

        // println!("\nCapture mask:");
        // _print_bitboard(capture_mask);

        // println!("\nPush mask:");
        // _print_bitboard(push_mask);

        // ======================================================================
        // ========================= Pinned piece setup =========================
        // ======================================================================

        let mut pinned_pieces: u64 = 0;
        let mut pinned_piece_moves: HashMap<u64, u64> = HashMap::new();

        {
            // Exclude the squares next to the king, since a sliding piece can't pin a piece to the king if it's next to the king.
            let mut possible_pinner_squares = self.get_queen_magic_attacks(king_square, 0)
                & !self.king_attacks[king_square as usize]
                & (self.bitboards[Piece::WhiteQueen as usize + (!self.turn as usize * 6)]
                    | self.bitboards[Piece::WhiteRook as usize + (!self.turn as usize * 6)]
                    | self.bitboards[Piece::WhiteBishop as usize + (!self.turn as usize * 6)]);

            // println!("\nPossible pinner squares:");
            // _print_bitboard(possible_pinner_squares);

            // For each of the possible pinner, check if there is a single friendly piece between
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

                let is_potential_bishop_pinner = (pinner_piece.is_bishop()
                    || pinner_piece.is_queen())
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
                let enemy_pieces_between_slider_and_king =
                    self.get_between_squares(king_square, pinner_square) & enemy_pieces;

                let friendly_pieces_between_slider_and_king =
                    self.get_between_squares(king_square, pinner_square) & friendly_pieces;

                if enemy_pieces_between_slider_and_king.count_ones() == 0
                    && friendly_pieces_between_slider_and_king.count_ones() == 1
                {
                    pinned_piece_moves.insert(
                        friendly_pieces_between_slider_and_king,
                        self.get_between_squares(king_square, pinner_square)
                            | (1u64 << pinner_square as u8),
                    );
                    pinned_pieces |= friendly_pieces_between_slider_and_king;
                }
            }

            // println!("\nPinned pieces:");
            // _print_bitboard(pinned_pieces);
        }

        // ======================================================================
        // ============================== Knight moves ==========================
        // ======================================================================

        {
            let mut movable_knights = self.bitboards
                [Piece::WhiteKnight as usize + (self.turn as usize * 6)]
                & !pinned_pieces;

            // println!("\nMovable knights:");
            // _print_bitboard(movable_knights);

            while movable_knights != 0 {
                let source = Square::from(utils::pop_lsb(&mut movable_knights));
                let knight_moves = self.knight_attacks[source as usize]
                    & !friendly_pieces
                    & (push_mask | capture_mask);

                // println!("\nKnight moves:");
                // _print_bitboard(knight_moves);

                self.append_bb_movelist(
                    knight_moves,
                    &mut moves,
                    Piece::from(Piece::WhiteKnight as usize + (self.turn as usize * 6)),
                    source,
                );
            }
        }

        // ======================================================================
        // ============================== Bishop moves ==========================
        // ======================================================================
        {
            let mut bishops =
                self.bitboards[Piece::WhiteBishop as usize + (self.turn as usize * 6)];

            while bishops != 0 {
                let source = Square::from(utils::pop_lsb(&mut bishops));
                let mut bishop_moves = self
                    .get_bishop_magic_attacks(source, self.get_both_occupancy())
                    & !friendly_pieces
                    & (push_mask | capture_mask);

                if pinned_pieces & (1u64 << source as u8) != 0 {
                    bishop_moves &= pinned_piece_moves[&(1u64 << source as u8)];
                }

                // println!("\nBishop moves:");
                // _print_bitboard(bishop_moves);

                self.append_bb_movelist(
                    bishop_moves,
                    &mut moves,
                    Piece::from(Piece::WhiteBishop as usize + (self.turn as usize * 6)),
                    source,
                );
            }
        }

        // ======================================================================
        // ============================== Rook moves ============================
        // ======================================================================
        {
            let mut rooks = self.bitboards[Piece::WhiteRook as usize + (self.turn as usize * 6)];

            while rooks != 0 {
                let source = Square::from(utils::pop_lsb(&mut rooks));
                let mut rook_moves = self.get_rook_magic_attacks(source, self.get_both_occupancy())
                    & !friendly_pieces
                    & (push_mask | capture_mask);

                if pinned_pieces & (1u64 << source as u8) != 0 {
                    rook_moves &= pinned_piece_moves[&(1u64 << source as u8)];
                }

                // println!("\nRook moves:");
                // _print_bitboard(rook_moves);

                self.append_bb_movelist(
                    rook_moves,
                    &mut moves,
                    Piece::from(Piece::WhiteRook as usize + (self.turn as usize * 6)),
                    source,
                );
            }
        }

        // ======================================================================
        // ============================== Queen moves ===========================
        // ======================================================================
        {
            let mut queens = self.bitboards[Piece::WhiteQueen as usize + (self.turn as usize * 6)];

            while queens != 0 {
                let source = Square::from(utils::pop_lsb(&mut queens));
                let mut queen_moves = self
                    .get_queen_magic_attacks(source, self.get_both_occupancy())
                    & !friendly_pieces
                    & (push_mask | capture_mask);

                if pinned_pieces & (1u64 << source as u8) != 0 {
                    queen_moves &= pinned_piece_moves[&(1u64 << source as u8)];
                }

                // println!("\nQueen moves:");
                // _print_bitboard(queen_moves);

                self.append_bb_movelist(
                    queen_moves,
                    &mut moves,
                    Piece::from(Piece::WhiteQueen as usize + (self.turn as usize * 6)),
                    source,
                );
            }
        }

        // ======================================================================
        // ============================== Pawn moves ============================
        // ======================================================================
        {
            let mut pawns = self.bitboards[Piece::WhitePawn as usize + (self.turn as usize * 6)];

            while pawns != 0 {
                let source = Square::from(utils::pop_lsb(&mut pawns));

                // Enpassant captures
                if let Some(enpassant_square) = self.enpassant {
                    let enpassant_attacks = self.pawn_attacks[self.turn as usize][source as usize]
                        & (1u64 << enpassant_square as u8);

                    if enpassant_attacks != 0 {
                        let target_enpassant = utils::get_lsb(enpassant_attacks);
                        let mut m = chess::_move::BitPackedMove::new(
                            source,
                            Square::from(target_enpassant),
                            Piece::from(Piece::WhitePawn as usize + (self.turn as usize * 6)),
                        );
                        m.set_enpassant();
                        moves.push(m);
                    }
                }

                // Captures
                let mut pawn_captures = self.pawn_attacks[self.turn as usize][source as usize]
                    & enemy_pieces
                    & (push_mask | capture_mask);

                if pinned_pieces & (1u64 << source as u8) != 0 {
                    pawn_captures &= pinned_piece_moves[&(1u64 << source as u8)];
                }

                // println!("\nPawn captures:");
                // _print_bitboard(pawn_captures);

                self.append_bb_movelist(
                    pawn_captures,
                    &mut moves,
                    Piece::from(Piece::WhitePawn as usize + (self.turn as usize * 6)),
                    source,
                );

                // Pushes
                if self.turn == Color::White {
                    let single_push = (1u64 << (source as u8 - 8))
                        & !(enemy_pieces | friendly_pieces)
                        & (push_mask);

                    let double_push =
                        (((1u64 << (source as u8 - 8)) & !(enemy_pieces | friendly_pieces)) >> 8)
                            & (push_mask)
                            & !(enemy_pieces | friendly_pieces)
                            & RANK_4;

                    let mut pawn_pushes = single_push | double_push;

                    if pinned_pieces & (1u64 << source as u8) != 0 {
                        pawn_pushes &= pinned_piece_moves[&(1u64 << source as u8)];
                    }

                    // println!("\nPawn pushes:");
                    // _print_bitboard(pawn_pushes);

                    self.append_bb_movelist(pawn_pushes, &mut moves, Piece::WhitePawn, source);
                } else {
                    let single_push = (1u64 << (source as u8 + 8))
                        & !(enemy_pieces | friendly_pieces)
                        & (push_mask);

                    let double_push =
                        (((1u64 << (source as u8 + 8)) & !(enemy_pieces | friendly_pieces)) << 8)
                            & (push_mask)
                            & !(enemy_pieces | friendly_pieces)
                            & RANK_5;

                    let mut pawn_pushes = single_push | double_push;

                    if pinned_pieces & (1u64 << source as u8) != 0 {
                        pawn_pushes &= pinned_piece_moves[&(1u64 << source as u8)];
                    }

                    // println!("\nPawn pushes:");
                    // _print_bitboard(pawn_pushes);

                    self.append_bb_movelist(pawn_pushes, &mut moves, Piece::BlackPawn, source);
                }
            }
        }

        return moves;
    }

    fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut nodes = 0;
        let moves = self.generate_legal_moves();

        for m in moves {
            let is_legal_move = self.make_move(m, false);

            if !is_legal_move {
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
        let moves = self.generate_legal_moves();
        println!("{}: {}", depth, moves.len());
        let mut nodes = 0;

        for m in moves {
            let is_legal_move = self.make_move(m, false);

            if !is_legal_move {
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

    fn detailed_perft(&mut self, depth: u8) -> PerftResult {
        let mut results = PerftResult {
            nodes: 1,
            captures: 0,
            enpassants: 0,
            castles: 0,
            promotions: 0,
            checkmates: 0,
            checks: 0,
            depth: 0,
            discovery_checks: 0,
            double_checks: 0,
        };

        if depth == 0 {
            return results;
        }

        let moves = self.generate_legal_moves();

        for m in moves {
            let is_legal_move = self.make_move(m, false);

            if !is_legal_move {
                if m.is_enpassant() {
                    continue;
                }
                self.draw();
                panic!("Illegal move: {}", m);
            }

            results = results + self.detailed_perft(depth - 1);

            self.unmake_move();
        }

        return results;
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
