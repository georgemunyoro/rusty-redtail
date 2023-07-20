use std::fmt::Display;

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
        constants::{RANK_2, RANK_7},
        piece::Piece,
        square::Square,
    },
    utils::{self, clear_bit, get_bit},
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

    fn perft(&mut self, depth: u8) -> u64;
}

#[derive(Debug, Clone, Copy)]
pub struct PerftResult {
    pub depth: u8,
    pub nodes: u64,
    pub captures: u64,
    pub en_passant: u64,
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
        result.push_str(format!("en_passant: {}\n", self.en_passant).as_str());
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
            en_passant: self.en_passant + other.en_passant,
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

    fn generate_legal_moves(&mut self) -> Vec<chess::_move::BitPackedMove> {
        let mut moves = Vec::with_capacity(256);

        for m in self.generate_moves(false) {
            let is_legal_move = self.make_move(m, false);
            if is_legal_move {
                moves.push(m);
                self.unmake_move();
            }
        }

        return moves;
    }

    fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut nodes = 0;
        let moves = self.generate_moves(false);

        for m in moves {
            let is_legal_move = self.make_move(m, false);
            if !is_legal_move {
                continue;
            }

            nodes += self.perft(depth - 1);
            self.unmake_move();
        }

        return nodes;
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
