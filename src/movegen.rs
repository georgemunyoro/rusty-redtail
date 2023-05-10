use std::fmt::Display;

use crate::{
    board::{Board, Position},
    chess, utils,
};

pub trait MoveGenerator {
    fn generate_moves(&self) -> Vec<chess::Move>;
    fn generate_legal_moves(&mut self) -> Vec<chess::Move>;

    fn generate_knight_moves(&self, move_list: &mut Vec<chess::Move>);
    fn generate_bishop_moves(&self, move_list: &mut Vec<chess::Move>);
    fn generate_rook_moves(&self, move_list: &mut Vec<chess::Move>);
    fn generate_queen_moves(&self, move_list: &mut Vec<chess::Move>);

    fn generate_king_moves(&self, move_list: &mut Vec<chess::Move>);
    fn generate_castle_moves(&self, move_list: &mut Vec<chess::Move>);

    fn generate_pawn_moves(&self, move_list: &mut Vec<chess::Move>);
    fn generate_white_pawn_moves(&self, move_list: &mut Vec<chess::Move>);
    fn generate_black_pawn_moves(&self, move_list: &mut Vec<chess::Move>);

    fn perft(&mut self, depth: u8) -> u64;
    fn perft_divide(&mut self, depth: u8) -> u64;
    fn detailed_perft(&mut self, depth: u8, print: bool) -> PerftResult;
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
    fn generate_black_pawn_moves(&self, moves: &mut Vec<chess::Move>) {
        let mut piece_bitboard =
            self.bitboards[chess::Piece::BlackPawn as usize] & !chess::constants::RANK_1;

        while piece_bitboard != 0 {
            let source = chess::Square::from(utils::get_lsb(piece_bitboard));
            let target = chess::Square::from((source as u8) + 8);

            // quiet pawn moves
            if (target <= chess::Square::H1)
                && self.get_piece_at_square(target as u8) == chess::Piece::Empty
            {
                // pawn promotion
                if source >= chess::Square::A2 && source <= chess::Square::H2 {
                    moves.extend(
                        vec![
                            chess::Piece::BlackQueen,
                            chess::Piece::BlackRook,
                            chess::Piece::BlackBishop,
                            chess::Piece::BlackKnight,
                        ]
                        .iter()
                        .map(|promotion| {
                            let mut m = chess::Move::new(source, target, chess::Piece::BlackPawn);
                            m.promotion = Some(*promotion);
                            return m;
                        }),
                    );
                } else {
                    // single pawn push
                    moves.push(chess::Move::new(source, target, chess::Piece::BlackPawn));

                    // double pawn push
                    if (source >= chess::Square::A7) && (source <= chess::Square::H7) {
                        let target = chess::Square::from((source as u8) + 16);
                        if self.get_piece_at_square(target as u8) == chess::Piece::Empty {
                            moves.push(chess::Move::new(source, target, chess::Piece::BlackPawn));
                        }
                    }
                }
            }

            // pawn captures
            let mut attacks = self.pawn_attacks[chess::Color::Black as usize][source as usize]
                & self.occupancies[chess::Color::White as usize];
            while attacks != 0 {
                let target = chess::Square::from(utils::get_lsb(attacks));

                if source >= chess::Square::A2 && source <= chess::Square::H2 {
                    moves.extend(
                        vec![
                            chess::Piece::BlackQueen,
                            chess::Piece::BlackRook,
                            chess::Piece::BlackBishop,
                            chess::Piece::BlackKnight,
                        ]
                        .iter()
                        .map(|promotion| {
                            let mut m = chess::Move::new(source, target, chess::Piece::BlackPawn);
                            m.promotion = Some(*promotion);
                            m.capture = Some(self.get_piece_at_square(target as u8));
                            return m;
                        }),
                    );
                } else {
                    let mut m = chess::Move::new(source, target, chess::Piece::BlackPawn);
                    m.capture = Some(self.get_piece_at_square(target as u8));
                    moves.push(m);
                }

                utils::clear_bit(&mut attacks, target as u8);
            }

            // generate enpassant captures
            match self.enpassant {
                None => {}
                _ => {
                    let enpassant_square = u8::from(self.enpassant.unwrap());
                    let enpassant_attacks = self.pawn_attacks[chess::Color::Black as usize]
                        [source as usize]
                        & (1u64 << enpassant_square);

                    if enpassant_attacks != 0 {
                        let target_enpassant = utils::get_lsb(enpassant_attacks);
                        let mut m = chess::Move::new(
                            source,
                            chess::Square::from(target_enpassant),
                            chess::Piece::BlackPawn,
                        );
                        m.en_passant = true;
                        moves.push(m);
                    }
                }
            }

            utils::pop_lsb(&mut piece_bitboard);
        }
    }

    fn generate_white_pawn_moves(&self, moves: &mut Vec<chess::Move>) {
        let mut piece_bitboard =
            self.bitboards[chess::Piece::WhitePawn as usize] & !chess::constants::RANK_8;

        while piece_bitboard != 0 {
            let source = chess::Square::from(utils::get_lsb(piece_bitboard));

            if source == chess::Square::A8 {
                println!("{} {} {}", source, source as usize, self.as_fen());
            }

            let target = chess::Square::from((source as usize) - 8);

            // quiet pawn moves
            if (target >= chess::Square::A8)
                && self.get_piece_at_square(target as u8) == chess::Piece::Empty
            {
                // pawn promotion
                if source >= chess::Square::A7 && source <= chess::Square::H7 {
                    moves.extend(
                        vec![
                            chess::Piece::WhiteQueen,
                            chess::Piece::WhiteRook,
                            chess::Piece::WhiteBishop,
                            chess::Piece::WhiteKnight,
                        ]
                        .iter()
                        .map(|promotion| {
                            let mut m = chess::Move::new(source, target, chess::Piece::WhitePawn);
                            m.promotion = Some(*promotion);
                            return m;
                        }),
                    );
                } else {
                    // single pawn push
                    moves.push(chess::Move::new(source, target, chess::Piece::WhitePawn));

                    // double pawn push
                    if (source >= chess::Square::A2) && (source <= chess::Square::H2) {
                        let target = chess::Square::from((source as u8) - 16);
                        if self.get_piece_at_square(target as u8) == chess::Piece::Empty {
                            moves.push(chess::Move::new(source, target, chess::Piece::WhitePawn));
                        }
                    }
                }
            }

            // pawn captures
            let mut attacks = self.pawn_attacks[chess::Color::White as usize][source as usize]
                & self.occupancies[chess::Color::Black as usize];
            while attacks != 0 {
                let target = chess::Square::from(utils::get_lsb(attacks));

                if source >= chess::Square::A7 && source <= chess::Square::H7 {
                    moves.extend(
                        vec![
                            chess::Piece::WhiteQueen,
                            chess::Piece::WhiteRook,
                            chess::Piece::WhiteBishop,
                            chess::Piece::WhiteKnight,
                        ]
                        .iter()
                        .map(|promotion| {
                            let mut m = chess::Move::new(source, target, chess::Piece::WhitePawn);
                            m.promotion = Some(*promotion);
                            m.capture = Some(self.get_piece_at_square(target as u8));
                            return m;
                        }),
                    );
                } else {
                    let mut m = chess::Move::new(source, target, chess::Piece::WhitePawn);
                    m.capture = Some(self.get_piece_at_square(target as u8));
                    moves.push(m);
                }

                utils::clear_bit(&mut attacks, target as u8);
            }

            // generate enpassant captures
            match self.enpassant {
                None => {}
                _ => {
                    let enpassant_square = u8::from(self.enpassant.unwrap());
                    let enpassant_attacks = self.pawn_attacks[chess::Color::White as usize]
                        [source as usize]
                        & (1u64 << enpassant_square);

                    if enpassant_attacks != 0 {
                        let target_enpassant = utils::get_lsb(enpassant_attacks);
                        let mut m = chess::Move::new(
                            source,
                            chess::Square::from(target_enpassant),
                            chess::Piece::WhitePawn,
                        );
                        m.en_passant = true;
                        moves.push(m);
                    }
                }
            }

            utils::pop_lsb(&mut piece_bitboard);
        }
    }

    fn generate_pawn_moves(&self, move_list: &mut Vec<chess::Move>) {
        // let mut moves = Vec::with_capacity(256);

        // white pawn moves
        if self.turn == chess::Color::White {
            self.generate_white_pawn_moves(move_list)
        }
        // black pawn moves
        else if self.turn == chess::Color::Black {
            self.generate_black_pawn_moves(move_list)
        }
    }

    fn generate_knight_moves(&self, moves: &mut Vec<chess::Move>) {
        let piece = if self.turn == chess::Color::White {
            chess::Piece::WhiteKnight
        } else {
            chess::Piece::BlackKnight
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut knight_moves = self.knight_attacks[i as usize] & !friendly_pieces;

            while knight_moves != 0 {
                let j = utils::pop_lsb(&mut knight_moves);
                let mut m = chess::Move::new(chess::Square::from(i), chess::Square::from(j), piece);

                let captured_piece = self.get_piece_at_square(j);
                if captured_piece != chess::Piece::Empty {
                    m.capture = Some(captured_piece);
                }

                moves.push(m);
            }
        }
    }

    fn generate_bishop_moves(&self, moves: &mut Vec<chess::Move>) {
        let piece = if self.turn == chess::Color::White {
            chess::Piece::WhiteBishop
        } else {
            chess::Piece::BlackBishop
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let enemy_pieces = self.occupancies[!self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut bishop_moves = self
                .get_bishop_magic_attacks(chess::Square::from(i), friendly_pieces | enemy_pieces)
                & !friendly_pieces;

            while bishop_moves != 0 {
                let j = utils::pop_lsb(&mut bishop_moves);
                let mut m = chess::Move::new(chess::Square::from(i), chess::Square::from(j), piece);

                let captured_piece = self.get_piece_at_square(j);
                if captured_piece != chess::Piece::Empty {
                    m.capture = Some(captured_piece);
                }

                moves.push(m);
            }
        }
    }

    fn generate_rook_moves(&self, moves: &mut Vec<chess::Move>) {
        let piece = if self.turn == chess::Color::White {
            chess::Piece::WhiteRook
        } else {
            chess::Piece::BlackRook
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let enemy_pieces = self.occupancies[!self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut rook_moves = self
                .get_rook_magic_attacks(chess::Square::from(i), friendly_pieces | enemy_pieces)
                & !friendly_pieces;

            while rook_moves != 0 {
                let j = utils::pop_lsb(&mut rook_moves);
                let mut m = chess::Move::new(chess::Square::from(i), chess::Square::from(j), piece);

                let captured_piece = self.get_piece_at_square(j);
                if captured_piece != chess::Piece::Empty {
                    m.capture = Some(captured_piece);
                }

                moves.push(m);
            }
        }
    }

    fn generate_queen_moves(&self, moves: &mut Vec<chess::Move>) {
        let piece = if self.turn == chess::Color::White {
            chess::Piece::WhiteQueen
        } else {
            chess::Piece::BlackQueen
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];
        let enemy_pieces = self.occupancies[!self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut queen_moves = self
                .get_queen_magic_attacks(chess::Square::from(i), friendly_pieces | enemy_pieces)
                & !friendly_pieces;

            while queen_moves != 0 {
                let j = utils::pop_lsb(&mut queen_moves);
                let mut m = chess::Move::new(chess::Square::from(i), chess::Square::from(j), piece);

                let captured_piece = self.get_piece_at_square(j);
                if captured_piece != chess::Piece::Empty {
                    m.capture = Some(captured_piece);
                }

                moves.push(m);
            }
        }
    }

    fn generate_king_moves(&self, moves: &mut Vec<chess::Move>) {
        let piece = if self.turn == chess::Color::White {
            chess::Piece::WhiteKing
        } else {
            chess::Piece::BlackKing
        };

        let mut piece_bitboard = self.bitboards[piece as usize];
        let friendly_pieces = self.occupancies[self.turn as usize];

        while piece_bitboard != 0 {
            let i = utils::pop_lsb(&mut piece_bitboard);
            let mut king_moves = self.king_attacks[i as usize] & !friendly_pieces;

            while king_moves != 0 {
                let j = utils::pop_lsb(&mut king_moves);
                let mut m = chess::Move::new(chess::Square::from(i), chess::Square::from(j), piece);

                let captured_piece = self.get_piece_at_square(j);
                if captured_piece != chess::Piece::Empty {
                    m.capture = Some(captured_piece);
                }

                moves.push(m);
            }
        }

        self.generate_castle_moves(moves);
    }

    fn generate_castle_moves(&self, moves: &mut Vec<chess::Move>) {
        if self.turn == chess::Color::White {
            let is_king_side_empty = self.get_piece_at_square(chess::Square::G1 as u8)
                == chess::Piece::Empty
                && self.get_piece_at_square(chess::Square::F1 as u8) == chess::Piece::Empty;

            if is_king_side_empty && self.castling.white_king_side {
                if !self.is_square_attacked(chess::Square::E1, chess::Color::Black)
                    && !self.is_square_attacked(chess::Square::F1, chess::Color::Black)
                    && self.get_piece_at_square(chess::Square::H1 as u8) == chess::Piece::WhiteRook
                {
                    let mut m = chess::Move::new(
                        chess::Square::E1,
                        chess::Square::G1,
                        chess::Piece::WhiteKing,
                    );
                    m.castle = true;
                    moves.push(m);
                }
            }

            let is_queen_side_empty = self.get_piece_at_square(chess::Square::D1 as u8)
                == chess::Piece::Empty
                && self.get_piece_at_square(chess::Square::C1 as u8) == chess::Piece::Empty
                && self.get_piece_at_square(chess::Square::B1 as u8) == chess::Piece::Empty;

            if is_queen_side_empty && self.castling.white_queen_side {
                if !self.is_square_attacked(chess::Square::E1, chess::Color::Black)
                    && !self.is_square_attacked(chess::Square::D1, chess::Color::Black)
                    && self.get_piece_at_square(chess::Square::A1 as u8) == chess::Piece::WhiteRook
                {
                    let mut m = chess::Move::new(
                        chess::Square::E1,
                        chess::Square::C1,
                        chess::Piece::WhiteKing,
                    );
                    m.castle = true;
                    moves.push(m);
                }
            }
        } else if self.turn == chess::Color::Black {
            let is_king_side_empty = self.get_piece_at_square(chess::Square::G8 as u8)
                == chess::Piece::Empty
                && self.get_piece_at_square(chess::Square::F8 as u8) == chess::Piece::Empty;

            if is_king_side_empty && self.castling.black_king_side {
                if !self.is_square_attacked(chess::Square::E8, chess::Color::White)
                    && !self.is_square_attacked(chess::Square::F8, chess::Color::White)
                    && self.get_piece_at_square(chess::Square::H8 as u8) == chess::Piece::BlackRook
                {
                    let mut m = chess::Move::new(
                        chess::Square::E8,
                        chess::Square::G8,
                        chess::Piece::BlackKing,
                    );
                    m.castle = true;
                    moves.push(m);
                }
            }

            let is_queen_side_empty = self.get_piece_at_square(chess::Square::D8 as u8)
                == chess::Piece::Empty
                && self.get_piece_at_square(chess::Square::C8 as u8) == chess::Piece::Empty
                && self.get_piece_at_square(chess::Square::B8 as u8) == chess::Piece::Empty;

            if is_queen_side_empty && self.castling.black_queen_side {
                if !self.is_square_attacked(chess::Square::E8, chess::Color::White)
                    && !self.is_square_attacked(chess::Square::D8, chess::Color::White)
                    && self.get_piece_at_square(chess::Square::A8 as u8) == chess::Piece::BlackRook
                {
                    let mut m = chess::Move::new(
                        chess::Square::E8,
                        chess::Square::C8,
                        chess::Piece::BlackKing,
                    );
                    m.castle = true;
                    moves.push(m);
                }
            }
        }
    }

    fn generate_moves(&self) -> Vec<chess::Move> {
        let mut moves = Vec::with_capacity(256);

        self.generate_pawn_moves(&mut moves);
        self.generate_knight_moves(&mut moves);
        self.generate_bishop_moves(&mut moves);
        self.generate_rook_moves(&mut moves);
        self.generate_queen_moves(&mut moves);
        self.generate_king_moves(&mut moves);

        return moves;
    }

    fn generate_legal_moves(&mut self) -> Vec<chess::Move> {
        let mut moves = Vec::with_capacity(256);

        for m in self.generate_moves() {
            let is_valid = self.make_move(m, false);

            if is_valid {
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
        let moves = self.generate_moves();

        for m in moves {
            let is_valid_move = self.make_move(m, false);
            if !is_valid_move {
                continue;
            }
            nodes += self.perft(depth - 1);
            self.unmake_move();
        }

        return nodes;
    }

    fn perft_divide(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut nodes = 0;
        let moves = self.generate_moves();

        for m in moves {
            let is_valid_move = self.make_move(m, false);
            if !is_valid_move {
                continue;
            }
            let child_nodes = self.perft(depth - 1);
            nodes += child_nodes;
            println!("{}: {}", m, child_nodes);
            self.unmake_move();
        }

        return nodes;
    }

    fn detailed_perft(&mut self, depth: u8, print: bool) -> PerftResult {
        if depth == 0 {
            return PerftResult {
                depth: depth,
                nodes: 1,
                captures: 0,
                en_passant: 0,
                castles: 0,
                promotions: 0,
                checks: 0,
                discovery_checks: 0,
                double_checks: 0,
                checkmates: 0,
            };
        }

        let mut result = PerftResult {
            depth: depth,
            nodes: 0,
            captures: 0,
            en_passant: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            discovery_checks: 0,
            double_checks: 0,
            checkmates: 0,
        };

        let moves = self.generate_moves();

        for m in moves {
            let is_valid_move = self.make_move(m, false);
            if !is_valid_move {
                continue;
            }

            let child_result = self.detailed_perft(depth - 1, false);

            result = result + child_result.clone();

            if print {
                println!("{}: {} - {}", m, child_result.nodes, self.as_fen());
            }
            self.unmake_move();
        }

        return result;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Position},
        chess,
        movegen::MoveGenerator,
    };

    #[test]
    fn test_starting_position_move_generation() {
        let mut board = Position::new(Some(String::from(chess::constants::STARTING_FEN).as_str()));

        let moves = board.generate_legal_moves();
        assert_eq!(moves.len(), 20);

        // Also check black's moves
        board.turn = chess::Color::Black;
        let black_moves = board.generate_moves();
        assert_eq!(black_moves.len(), 20);
    }
}
