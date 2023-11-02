pub mod attacks;
pub mod constants;

use std::collections::HashMap;

use crate::{
    chess::{
        self,
        castling_rights::CastlingRights,
        color::Color,
        piece::{Piece, PieceTrait},
        square::Square,
        square::SQUARE_ITER,
    },
    pst::{
        END_BISHOP_POSITIONAL_SCORE, END_KING_POSITIONAL_SCORE, END_KNIGHT_POSITIONAL_SCORE,
        END_PAWN_POSITIONAL_SCORE, END_QUEEN_POSITIONAL_SCORE, END_ROOK_POSITIONAL_SCORE,
        MIRROR_SCORE, OPEN_BISHOP_POSITIONAL_SCORE, OPEN_KING_POSITIONAL_SCORE,
        OPEN_KNIGHT_POSITIONAL_SCORE, OPEN_PAWN_POSITIONAL_SCORE, OPEN_QUEEN_POSITIONAL_SCORE,
        OPEN_ROOK_POSITIONAL_SCORE,
    },
    utils::{self, get_bit, pop_lsb},
};

use self::attacks::{
    KING_ATTACKS, KNIGHT_ATTACKS, MAGIC_BISHOP_ATTACKS, MAGIC_BISHOP_MASKS, MAGIC_ROOK_ATTACKS,
    MAGIC_ROOK_MASKS, PAWN_ATTACKS,
};

/// A chess position
pub struct Position {
    pub bitboards: [u64; 12],
    pub turn: Color,
    pub enpassant: Option<Square>,
    pub castling: CastlingRights,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub position_stack: Vec<HistoryEntry>,
    pub occupancies: [u64; 3],
    pub hash: u64,
    pub material: [i32; 2],
    pub move_list_stack: Vec<Vec<chess::_move::BitPackedMove>>,
    pub depth: usize,

    pub pinner_map: HashMap<u64, u64>,

    pub rand_seed: u32,

    pub zobrist_piece_keys: [[u64; 64]; 12],
    pub zobrist_castling_keys: [u64; 16],
    pub zobrist_enpassant_keys: [u64; 64],
    pub zobrist_turn_key: u64,
    pub opponent_attack_map: u64,
}

#[derive(Clone, Copy)]
pub struct HistoryEntry {
    pub bitboards: [u64; 12],
    pub turn: Color,
    pub enpassant: Option<Square>,
    pub castling: CastlingRights,
    pub material: [i32; 2],

    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub occupancies: [u64; 3],
    pub hash: u64,
}

impl HistoryEntry {
    fn default() -> HistoryEntry {
        return HistoryEntry {
            bitboards: [0; 12],
            turn: Color::White,
            enpassant: None,
            castling: CastlingRights::new(),
            material: [0; 2],
            halfmove_clock: 0,
            fullmove_number: 1,
            occupancies: [0; 3],
            hash: 0,
        };
    }
}

pub trait Board {
    fn new(fen: Option<&str>) -> Position;
    fn draw(&mut self);
    fn set_fen(&mut self, fen: String);
    fn is_square_attacked(&self, square: Square, color: Color) -> bool;
    fn is_square_attacked_w_occupancy(
        &self,
        square: Square,
        color: Color,
        both_occupancy: u64,
    ) -> bool;

    fn get_game_phase_score(&self) -> i32;

    fn is_in_check(&self) -> bool;

    fn make_move(&mut self, m: &chess::_move::BitPackedMove, only_captures: bool) -> bool;
    fn unmake_move(&mut self);
    fn make_null_move(&mut self);

    fn as_fen(&self) -> String;
}

impl Board for Position {
    fn make_null_move(&mut self) {
        // add the move to the history
        let history_entry = self.to_history_entry();
        self.position_stack.push(history_entry);
        self.depth += 1;

        self.turn = !self.turn;
        self.hash ^= self.zobrist_turn_key;
    }

    fn new(fen: Option<&str>) -> Position {
        let mut pos = Position {
            bitboards: [0; 12],
            turn: Color::White,

            enpassant: None,
            castling: CastlingRights::new(),
            halfmove_clock: 0,
            fullmove_number: 1,

            rand_seed: 1804289383,

            position_stack: vec![HistoryEntry::default(); 128],

            occupancies: [0; 3],

            zobrist_piece_keys: [[0; 64]; 12],
            zobrist_castling_keys: [0; 16],
            zobrist_enpassant_keys: [0; 64],
            zobrist_turn_key: 0,

            hash: 0,
            material: [0, 0],

            opponent_attack_map: 0,

            move_list_stack: vec![Vec::with_capacity(256); 128],
            depth: 0,

            pinner_map: HashMap::with_capacity(64),
        };

        pos.init_zorbrist_keys();
        pos.update_occupancies();
        pos.update_hash();

        match fen {
            Some(p) => Position::set_fen(&mut pos, String::from(p)),
            None => {}
        }

        return pos;
    }

    /// Returns true if the current side's king is in check
    fn is_in_check(&self) -> bool {
        let king_square =
            utils::get_lsb(self.bitboards[(self.turn as usize * 6) + (Piece::WhiteKing as usize)]);
        return king_square >= 64 || self.is_square_attacked(Square::from(king_square), !self.turn);
    }

    /// Returns an FEN string representing the current position
    fn as_fen(&self) -> String {
        let mut fen = String::new();

        // Piece placement
        let mut empty = 0;
        for i in 0..64 {
            if i % 8 == 0 && i != 0 {
                if empty != 0 {
                    fen.push_str(&empty.to_string());
                    empty = 0;
                }
                fen.push('/');
            }

            if get_bit(self.occupancies[2], i) == 0 {
                empty += 1;
            } else {
                let piece = self.get_piece_at_square(i);
                if empty != 0 {
                    fen.push_str(&empty.to_string());
                    empty = 0;
                }
                fen.push_str(&piece.to_string());
            }
        }
        if empty != 0 {
            fen.push_str(&empty.to_string());
        }

        // Active color
        fen.push(' ');
        fen.push_str(match self.turn {
            Color::White => "w",
            Color::Black => "b",
        });

        // Castling availability
        fen.push(' ');
        if self.castling.can_castle(CastlingRights::WHITE_KINGSIDE) {
            fen.push('K');
        }
        if self.castling.can_castle(CastlingRights::WHITE_QUEENSIDE) {
            fen.push('Q');
        }
        if self.castling.can_castle(CastlingRights::BLACK_KINGSIDE) {
            fen.push('k');
        }
        if self.castling.can_castle(CastlingRights::BLACK_QUEENSIDE) {
            fen.push('q');
        }
        if self.castling.get_rights_u8() == 0 {
            fen.push('-');
        }

        // En passant target square
        fen.push(' ');
        match self.enpassant {
            Some(s) => fen.push_str(&s.to_string().to_lowercase()),
            None => fen.push('-'),
        }

        // Halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());

        // Fullmove number
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        return fen;
    }

    /// Draws the board to the console
    fn draw(&mut self) {
        for i in 0..64 {
            if i % 8 == 0 {
                println!();
            }
            print!("{} ", self.get_piece_at_square(i));
        }
        println!("\n\n{} to move", self.turn);
        println!("{}", self.as_fen());
        println!("HASH: {:016x}", self.hash);
    }

    /// Sets the board to the given FEN string
    fn set_fen(&mut self, fen: String) {
        // Split the fen
        let sections = fen.split(' ').collect::<Vec<&str>>();

        // Clear the bitboards
        for i in 0..12 {
            self.bitboards[i] = 0;
        }

        // Set the bitboard positions
        let mut pos = 0;
        for c in sections[0].chars() {
            if c == '/' {
                continue;
            }

            if c.is_digit(10) {
                pos += c.to_digit(10).unwrap() as u8;
                continue;
            }

            utils::set_bit(&mut self.bitboards[Piece::from(c) as usize], pos);
            pos += 1;
        }

        for i in 0..64 {
            let piece = self.get_piece_at_square(i);
            if piece == chess::piece::Piece::Empty {
                continue;
            }
            let v = _get_piece_value_bl(piece as usize, i as usize, self.get_game_phase_score());
            if piece as usize >= 6 {
                self.material[Color::Black as usize] += v;
            } else {
                self.material[Color::White as usize] += v;
            }
        }

        // Set the turn
        self.turn = Color::from(sections[1].chars().next().unwrap());

        // Set the castling rights
        self.castling = CastlingRights::from(sections[2]);

        // Set the en passant square
        self.enpassant = match sections[3] {
            "-" => None,
            _ => Some(Square::from(sections[3])),
        };

        // Set the halfmove clock
        self.halfmove_clock = sections[4].parse::<u32>().unwrap();

        // Set the fullmove number
        self.fullmove_number = sections[5].parse::<u32>().unwrap();

        self.update_occupancies();
        self.update_hash();
    }

    /// Returns true if the given square is attacked by the given color, takes a custom occupancy
    fn is_square_attacked_w_occupancy(
        &self,
        square: Square,
        color: Color,
        both_occupancy: u64,
    ) -> bool {
        let piece_offset = color as usize * 6;

        let p_attacks = PAWN_ATTACKS[!color as usize][square as usize]
            & self.bitboards[Piece::WhitePawn as usize + piece_offset];

        let n_attacks = KNIGHT_ATTACKS[square as usize]
            & self.bitboards[Piece::WhiteKnight as usize + piece_offset];

        let bq_attacks = self.get_bishop_magic_attacks(square, both_occupancy)
            & (self.bitboards[Piece::WhiteBishop as usize + piece_offset]
                | self.bitboards[Piece::WhiteQueen as usize + piece_offset]);

        let rq_attacks = self.get_rook_magic_attacks(square, both_occupancy)
            & (self.bitboards[Piece::WhiteRook as usize + piece_offset]
                | self.bitboards[Piece::WhiteQueen as usize + piece_offset]);

        let k_attacks = KING_ATTACKS[square as usize]
            & self.bitboards[Piece::WhiteKing as usize + piece_offset];

        return (p_attacks | n_attacks | bq_attacks | rq_attacks | k_attacks) != 0;
    }

    /// Returns true if the given square is attacked by the given color
    fn is_square_attacked(&self, square: Square, color: Color) -> bool {
        return self.is_square_attacked_w_occupancy(square, color, self.occupancies[2]);
    }

    fn get_game_phase_score(&self) -> i32 {
        (utils::count_bits(
            self.bitboards[Piece::BlackBishop as usize]
                | self.bitboards[Piece::WhiteBishop as usize],
        ) as i32
            * 365)
            + (utils::count_bits(
                self.bitboards[Piece::BlackKnight as usize]
                    | self.bitboards[Piece::WhiteKnight as usize],
            ) as i32
                * 337)
            + (utils::count_bits(
                self.bitboards[Piece::BlackRook as usize]
                    | self.bitboards[Piece::WhiteRook as usize],
            ) as i32
                * 477)
            + (utils::count_bits(
                self.bitboards[Piece::WhiteQueen as usize]
                    | self.bitboards[Piece::BlackQueen as usize],
            ) as i32
                * 1025)
    }

    fn make_move(&mut self, m: &chess::_move::BitPackedMove, only_captures: bool) -> bool {
        if only_captures && !m.is_capture() {
            return false;
        }

        let piece_usize = m.get_piece_usize();
        let piece = Piece::from(piece_usize);
        let to = m.get_to();
        let to_usize = to as usize;
        let from = m.get_from();
        let from_usize = from as usize;

        // Add the move to the history
        {
            let entry = &mut self.position_stack[self.depth];

            self.bitboards.clone_into(&mut entry.bitboards);
            self.turn.clone_into(&mut entry.turn);
            self.enpassant.clone_into(&mut entry.enpassant);
            self.castling.clone_into(&mut entry.castling);

            entry.occupancies = self.occupancies;
            entry.material = self.material;
            entry.hash = self.hash;
            entry.halfmove_clock = self.halfmove_clock;
            // entry.fullmove_number = self.fullmove_number;
        }

        // set the moving piece
        utils::set_bit(&mut self.bitboards[piece_usize], to_usize as u8);

        let mut xor_mask = 0u64;

        xor_mask ^= self.zobrist_piece_keys[piece_usize][from_usize];
        xor_mask ^= self.zobrist_piece_keys[piece_usize][to_usize];

        // remove the moving piece
        utils::clear_bit(&mut self.bitboards[piece_usize], from_usize as u8);

        // handle captures
        if m.is_capture() {
            let captured_piece = m.get_capture();
            // remove the captured piece
            utils::clear_bit(&mut self.bitboards[captured_piece as usize], to_usize as u8);
            xor_mask ^= self.zobrist_piece_keys[captured_piece as usize][to_usize];
        }

        if let Some(enpassant_square) = self.enpassant {
            xor_mask ^= self.zobrist_enpassant_keys[enpassant_square as usize];
        }

        self.enpassant = None;

        if piece.is_pawn() {
            // Handle promotion
            if m.is_promotion() {
                // remove the pawn
                utils::clear_bit(&mut self.bitboards[piece_usize], to_usize as u8);
                xor_mask ^= self.zobrist_piece_keys[piece_usize][to_usize];

                // add the promoted piece
                utils::set_bit(
                    &mut self.bitboards[m.get_promotion() as usize],
                    to_usize as u8,
                );
                xor_mask ^= self.zobrist_piece_keys[m.get_promotion() as usize][to_usize];
            }

            // Handle en passant moves
            if m.is_enpassant() {
                let en_captured_square = ((to_usize as i8) + (8i8 - (self.turn as i8 * 16))) as u8;

                if get_bit(self.occupancies[2], en_captured_square) != 0 {
                    let en_captured_piece = self.get_piece_at_square(en_captured_square);

                    // remove the captured pawn
                    utils::clear_bit(
                        &mut self.bitboards[en_captured_piece as usize],
                        en_captured_square,
                    );
                }

                self.update_occupancies();

                // This is for en passant check evasion, computationally
                // cheaper to do it here than in the move generator
                if self.is_in_check() {
                    self.depth += 1;
                    self.unmake_move();
                    return false;
                }
            }

            // Set the en passant square during double pawn pushes
            let offset = -1 * (from_usize as i8 - to_usize as i8) / 2;
            if offset.abs() == 8 {
                self.enpassant = Some(Square::from((from_usize as i8 + offset) as u8));
                xor_mask ^= self.zobrist_enpassant_keys[(from_usize as i8 + offset) as usize];
            } else {
                self.enpassant = None;
            }
        }

        if piece.is_king() {
            // handle castling
            if m.is_castle() {
                match to {
                    Square::C1 => {
                        // white queen side
                        utils::clear_bit(
                            &mut self.bitboards[Piece::WhiteRook as usize],
                            Square::A1 as u8,
                        );
                        xor_mask ^=
                            self.zobrist_piece_keys[Piece::WhiteRook as usize][Square::A1 as usize];

                        utils::set_bit(
                            &mut self.bitboards[Piece::WhiteRook as usize],
                            Square::D1 as u8,
                        );

                        xor_mask ^=
                            self.zobrist_piece_keys[Piece::WhiteRook as usize][Square::D1 as usize];
                    }
                    Square::G1 => {
                        // white king side
                        utils::clear_bit(
                            &mut self.bitboards[Piece::WhiteRook as usize],
                            Square::H1 as u8,
                        );

                        xor_mask ^=
                            self.zobrist_piece_keys[Piece::WhiteRook as usize][Square::H1 as usize];

                        utils::set_bit(
                            &mut self.bitboards[Piece::WhiteRook as usize],
                            Square::F1 as u8,
                        );

                        xor_mask ^=
                            self.zobrist_piece_keys[Piece::WhiteRook as usize][Square::F1 as usize];
                    }
                    Square::C8 => {
                        // black queen side
                        utils::clear_bit(
                            &mut self.bitboards[Piece::BlackRook as usize],
                            Square::A8 as u8,
                        );

                        xor_mask ^=
                            self.zobrist_piece_keys[Piece::BlackRook as usize][Square::A8 as usize];

                        utils::set_bit(
                            &mut self.bitboards[Piece::BlackRook as usize],
                            Square::D8 as u8,
                        );

                        xor_mask ^=
                            self.zobrist_piece_keys[Piece::BlackRook as usize][Square::D8 as usize];
                    }
                    Square::G8 => {
                        // black king side
                        utils::clear_bit(
                            &mut self.bitboards[Piece::BlackRook as usize],
                            Square::H8 as u8,
                        );

                        xor_mask ^=
                            self.zobrist_piece_keys[Piece::BlackRook as usize][Square::H8 as usize];

                        utils::set_bit(
                            &mut self.bitboards[Piece::BlackRook as usize],
                            Square::F8 as u8,
                        );

                        xor_mask ^=
                            self.zobrist_piece_keys[Piece::BlackRook as usize][Square::F8 as usize];
                    }
                    _ => {}
                }
            }

            xor_mask ^= self.zobrist_castling_keys[self.castling.get_rights_u8() as usize];

            if piece == Piece::WhiteKing {
                self.castling
                    .remove_right(CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE);
            } else if piece == Piece::BlackKing {
                self.castling
                    .remove_right(CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE);
            }
        }

        if piece.is_rook() {
            match piece {
                Piece::WhiteRook => {
                    if from == Square::A1 {
                        self.castling.remove_right(CastlingRights::WHITE_QUEENSIDE);
                    } else if from == Square::H1 {
                        self.castling.remove_right(CastlingRights::WHITE_KINGSIDE);
                    }
                }
                Piece::BlackRook => {
                    if from == Square::A8 {
                        self.castling.remove_right(CastlingRights::BLACK_QUEENSIDE);
                    } else if from == Square::H8 {
                        self.castling.remove_right(CastlingRights::BLACK_KINGSIDE);
                    }
                }
                _ => {}
            }
        }

        xor_mask ^= self.zobrist_castling_keys[self.castling.get_rights_u8() as usize];
        self.update_occupancies();
        self.depth += 1;

        self.turn = !self.turn;
        xor_mask ^= self.zobrist_turn_key;

        self.hash ^= xor_mask;

        return true;
    }

    fn unmake_move(&mut self) {
        self.depth -= 1;

        let entry = &self.position_stack[self.depth];
        entry.bitboards.clone_into(&mut self.bitboards);
        entry.occupancies.clone_into(&mut self.occupancies);
        entry.turn.clone_into(&mut self.turn);
        entry.enpassant.clone_into(&mut self.enpassant);
        entry.castling.clone_into(&mut self.castling);
        entry.material.clone_into(&mut self.material);
        entry.hash.clone_into(&mut self.hash);
        entry.halfmove_clock.clone_into(&mut self.halfmove_clock);
        entry.fullmove_number.clone_into(&mut self.fullmove_number);
    }
}

impl Position {
    pub fn update_hash(&mut self) {
        let mut hash: u64 = 0;
        for piece in 0..12 {
            let mut bb = self.bitboards[piece];
            while bb != 0 {
                let square = pop_lsb(&mut bb);
                hash ^= self.zobrist_piece_keys[piece][square as usize];
            }
        }

        if self.enpassant != None {
            hash ^= self.zobrist_enpassant_keys[self.enpassant.unwrap() as usize];
        }

        hash ^= self.zobrist_castling_keys[self.castling.get_rights_u8() as usize];

        if self.turn == Color::Black {
            hash ^= self.zobrist_turn_key;
        }

        self.hash = hash;
    }

    pub fn init_zorbrist_keys(&mut self) {
        self.rand_seed = 1804289383;

        for piece in (Piece::WhitePawn as usize)..=(Piece::BlackKing as usize) {
            for square in 0..64 {
                self.zobrist_piece_keys[piece as usize][square] =
                    utils::get_pseudorandom_number_u64(&mut self.rand_seed);
            }
        }

        for square in 0..64 {
            self.zobrist_enpassant_keys[square] =
                utils::get_pseudorandom_number_u64(&mut self.rand_seed);
        }

        for i in 0..16 {
            self.zobrist_castling_keys[i] = utils::get_pseudorandom_number_u64(&mut self.rand_seed);
        }

        self.zobrist_turn_key = utils::get_pseudorandom_number_u64(&mut self.rand_seed);
    }

    pub fn to_history_entry(&self) -> HistoryEntry {
        return HistoryEntry {
            bitboards: self.bitboards,
            turn: self.turn,
            enpassant: self.enpassant,
            castling: self.castling,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
            occupancies: self.occupancies,
            material: self.material,
            hash: self.hash,
        };
    }

    /// updates the occupancies bitboards
    fn update_occupancies(&mut self) {
        unsafe {
            let white_occupancy = self.bitboards.get_unchecked(0)
                | self.bitboards.get_unchecked(1)
                | self.bitboards.get_unchecked(2)
                | self.bitboards.get_unchecked(3)
                | self.bitboards.get_unchecked(4)
                | self.bitboards.get_unchecked(5);
            let black_occupancy = self.bitboards.get_unchecked(6)
                | self.bitboards.get_unchecked(7)
                | self.bitboards.get_unchecked(8)
                | self.bitboards.get_unchecked(9)
                | self.bitboards.get_unchecked(10)
                | self.bitboards.get_unchecked(11);

            *self.occupancies.get_unchecked_mut(0) = white_occupancy;
            *self.occupancies.get_unchecked_mut(1) = black_occupancy;
            *self.occupancies.get_unchecked_mut(2) = white_occupancy | black_occupancy;
        }
    }

    pub fn apply_history_entry(&mut self, entry: HistoryEntry) {
        entry.bitboards.clone_into(&mut self.bitboards);
        self.turn = entry.turn;
        self.enpassant = entry.enpassant;
        self.castling = entry.castling;
        self.halfmove_clock = entry.halfmove_clock;
        self.fullmove_number = entry.fullmove_number;
        self.occupancies = entry.occupancies;
        self.material = entry.material;
        self.hash = entry.hash;
    }

    pub fn get_piece_at_square(&self, square: u8) -> Piece {
        static BLACK_PIECES: [Piece; 6] = [
            Piece::BlackPawn,
            Piece::BlackKnight,
            Piece::BlackBishop,
            Piece::BlackRook,
            Piece::BlackQueen,
            Piece::BlackKing,
        ];

        static WHITE_PIECES: [Piece; 6] = [
            Piece::WhitePawn,
            Piece::WhiteKnight,
            Piece::WhiteBishop,
            Piece::WhiteRook,
            Piece::WhiteQueen,
            Piece::WhiteKing,
        ];

        if utils::get_bit(self.occupancies[Color::White as usize], square) != 0 {
            for piece in WHITE_PIECES {
                if utils::get_bit(self.bitboards[piece as usize], square) != 0 {
                    return piece;
                }
            }
        } else {
            for piece in BLACK_PIECES {
                if utils::get_bit(self.bitboards[piece as usize], square) != 0 {
                    return piece;
                }
            }
        };

        return Piece::Empty;
    }

    fn mask_bishop_attacks(&self, square: Square) -> u64 {
        let mut attacks: u64 = 0;

        let square_rank = i8::from(square) / 8;
        let square_file = i8::from(square) % 8;

        for (rank, file) in ((square_rank + 1)..=6).zip((square_file + 1)..=6) {
            utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());
        }

        for (rank, file) in (1..=(square_rank - 1))
            .rev()
            .zip((1..=(square_file - 1)).rev())
        {
            utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());
        }

        for (rank, file) in ((square_rank + 1)..=6).zip((1..=(square_file - 1)).rev()) {
            utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());
        }

        for (rank, file) in (1..=(square_rank - 1)).rev().zip((square_file + 1)..=6) {
            utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());
        }

        return attacks;
    }

    fn mask_rook_attacks(&self, square: Square) -> u64 {
        let mut attacks: u64 = 0;

        let square_rank = i8::from(square) / 8;
        let square_file = i8::from(square) % 8;

        for rank in (square_rank + 1)..=6 {
            utils::set_bit(&mut attacks, (rank * 8 + square_file).try_into().unwrap());
        }

        for rank in 1..=(square_rank - 1) {
            utils::set_bit(&mut attacks, (rank * 8 + square_file).try_into().unwrap());
        }

        for file in (square_file + 1)..=6 {
            utils::set_bit(&mut attacks, (square_rank * 8 + file).try_into().unwrap());
        }

        for file in 1..=(square_file - 1) {
            utils::set_bit(&mut attacks, (square_rank * 8 + file).try_into().unwrap());
        }

        return attacks;
    }

    fn generate_bishop_attacks_on_the_fly(&self, square: Square, blockers: u64) -> u64 {
        let mut attacks: u64 = 0;

        let square_rank = i8::from(square) / 8;
        let square_file = i8::from(square) % 8;

        for (rank, file) in ((square_rank + 1)..=7).zip((square_file + 1)..=7) {
            utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());

            if ((1u64 << (rank * 8 + file)) & blockers) != 0 {
                break;
            }
        }

        for (rank, file) in (0..=(square_rank - 1))
            .rev()
            .zip((0..=(square_file - 1)).rev())
        {
            utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());

            if ((1u64 << (rank * 8 + file)) & blockers) != 0 {
                break;
            }
        }

        for (rank, file) in ((square_rank + 1)..=7).zip((0..=(square_file - 1)).rev()) {
            utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());

            if ((1u64 << (rank * 8 + file)) & blockers) != 0 {
                break;
            }
        }

        for (rank, file) in (0..=(square_rank - 1)).rev().zip((square_file + 1)..=7) {
            utils::set_bit(&mut attacks, (rank * 8 + file).try_into().unwrap());

            if ((1u64 << (rank * 8 + file)) & blockers) != 0 {
                break;
            }
        }

        return attacks;
    }

    fn generate_rook_attacks_on_the_fly(&self, square: Square, blockers: u64) -> u64 {
        let mut attacks: u64 = 0;

        let square_rank = i8::from(square) / 8;
        let square_file = i8::from(square) % 8;

        for rank in (square_rank + 1)..=7 {
            utils::set_bit(&mut attacks, (rank * 8 + square_file).try_into().unwrap());

            if ((1u64 << (rank * 8 + square_file)) & blockers) != 0 {
                break;
            }
        }

        for rank in (0..=(square_rank - 1)).rev() {
            utils::set_bit(&mut attacks, (rank * 8 + square_file).try_into().unwrap());

            if ((1u64 << (rank * 8 + square_file)) & blockers) != 0 {
                break;
            }
        }

        for file in (square_file + 1)..=7 {
            utils::set_bit(&mut attacks, (square_rank * 8 + file).try_into().unwrap());

            if ((1u64 << (square_rank * 8 + file)) & blockers) != 0 {
                break;
            }
        }

        for file in (0..=(square_file - 1)).rev() {
            utils::set_bit(&mut attacks, (square_rank * 8 + file).try_into().unwrap());

            if ((1u64 << (square_rank * 8 + file)) & blockers) != 0 {
                break;
            }
        }

        return attacks;
    }

    fn set_occupancy(&self, index: u32, bits_in_mask: u64, attack_mask: u64) -> u64 {
        let mut occupancy = 0u64;
        let mut mutable_attack_mask = attack_mask;

        for i in 0..bits_in_mask {
            let square = utils::get_lsb(mutable_attack_mask);
            utils::clear_bit(&mut mutable_attack_mask, square);

            if (index & (1 << i)) != 0 {
                utils::set_bit(&mut occupancy, square);
            }
        }

        return occupancy;
    }

    fn _generate_magic_number(&mut self) -> u64 {
        return utils::get_pseudorandom_number_u64(&mut self.rand_seed)
            & utils::get_pseudorandom_number_u64(&mut self.rand_seed)
            & utils::get_pseudorandom_number_u64(&mut self.rand_seed);
    }

    fn _find_magic_number(&mut self, square: Square, relevant_bits: u32, is_bishop: bool) -> u64 {
        let mut occupancies: [u64; 4096] = [0u64; 4096];
        let mut attacks: [u64; 4096] = [0u64; 4096];
        let mut _used_attacks: Vec<u64> = vec![0u64; std::mem::size_of::<u64>() * 4096];

        let attack_mask = if is_bishop {
            self.mask_bishop_attacks(square)
        } else {
            self.mask_rook_attacks(square)
        };

        let occupancy_indices = 1 << relevant_bits;

        for i in 0..occupancy_indices {
            occupancies[i as usize] = self.set_occupancy(i, relevant_bits as u64, attack_mask);
            attacks[i as usize] = if is_bishop {
                self.generate_bishop_attacks_on_the_fly(square, occupancies[i as usize])
            } else {
                self.generate_rook_attacks_on_the_fly(square, occupancies[i as usize])
            };
        }

        for _ in 0..10000000 {
            let magic_number = self._generate_magic_number();
            if utils::count_bits((attack_mask.wrapping_mul(magic_number)) & 0xFF00000000000000) < 6
            {
                continue;
            }

            _used_attacks = vec![0u64; std::mem::size_of::<u64>() * 4096];

            let (mut index, mut fail): (u32, bool) = (0, false);

            while index < occupancy_indices && !fail {
                let magic_index = ((occupancies[index as usize].wrapping_mul(magic_number))
                    >> (64 - relevant_bits)) as usize;

                if _used_attacks[magic_index] == 0 {
                    _used_attacks[magic_index] = attacks[index as usize];
                } else if _used_attacks[magic_index] != attacks[index as usize] {
                    fail = true;
                }

                index += 1;
            }

            if !fail {
                return magic_number;
            }
        }

        return 0u64;
    }

    fn _initialize_magic_numbers(&mut self) {
        println!("--------------------------------");
        println!("Generating magic numbers...");
        println!("--------------------------------\n");

        for square in SQUARE_ITER {
            let magic_number = self._find_magic_number(
                square,
                constants::ROOK_RELEVANT_BITS[usize::from(u8::from(square))],
                false,
            );

            println!("Bishop {} 0x{:016x}u64", square, magic_number);
        }

        println!("--------------------------------");

        for square in SQUARE_ITER {
            let magic_number = self._find_magic_number(
                square,
                constants::BISHOP_RELEVANT_BITS[usize::from(u8::from(square))],
                true,
            );

            println!("Rook {} 0x{:016x}u64", square, magic_number);
        }
    }

    pub fn get_bishop_magic_attacks(&self, square: Square, occupancy: u64) -> u64 {
        unsafe {
            let mut mutable_occupancy = occupancy;
            mutable_occupancy &= MAGIC_BISHOP_MASKS.get_unchecked(square as usize);
            mutable_occupancy = mutable_occupancy
                .wrapping_mul(*constants::BISHOP_MAGIC_NUMBERS.get_unchecked(square as usize));
            mutable_occupancy >>=
                64 - constants::BISHOP_RELEVANT_BITS.get_unchecked(square as usize);
            return *MAGIC_BISHOP_ATTACKS
                .get_unchecked(square as usize)
                .get_unchecked(mutable_occupancy as usize);
        }
    }

    pub fn get_rook_magic_attacks(&self, square: Square, occupancy: u64) -> u64 {
        unsafe {
            let mut mutable_occupancy = occupancy;
            mutable_occupancy &= MAGIC_ROOK_MASKS.get_unchecked(square as usize);
            mutable_occupancy = mutable_occupancy
                .wrapping_mul(*constants::ROOK_MAGIC_NUMBERS.get_unchecked(square as usize));
            mutable_occupancy >>= 64 - constants::ROOK_RELEVANT_BITS.get_unchecked(square as usize);
            return *MAGIC_ROOK_ATTACKS
                .get_unchecked(square as usize)
                .get_unchecked(mutable_occupancy as usize);
        }
    }

    pub fn get_queen_magic_attacks(&self, square: Square, occupancy: u64) -> u64 {
        return self.get_bishop_magic_attacks(square, occupancy)
            | self.get_rook_magic_attacks(square, occupancy);
    }

    pub fn get_both_occupancy(&self) -> u64 {
        let mut white_occupancy = 0u64;
        let mut black_occupancy = 0u64;

        for piece in (Piece::WhitePawn as u8)..=(Piece::WhiteKing as u8) {
            white_occupancy |= self.bitboards[piece as usize];
        }

        for piece in (Piece::BlackPawn as u8)..=(Piece::BlackKing as u8) {
            black_occupancy |= self.bitboards[piece as usize];
        }

        return white_occupancy | black_occupancy;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{constants, Board, Position},
        chess::{constants::STARTING_FEN, piece::Piece, square::SQUARE_ITER},
        movegen::MoveGenerator,
    };

    extern crate test;

    use test::{black_box, Bencher};

    use super::_get_piece_value_bl;

    #[test]
    fn get_piece_at_square_and_set_fen_works() {
        let position = Position::new(Some(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ));
        assert_eq!(position.get_piece_at_square(0), Piece::BlackRook);
        assert_eq!(position.get_piece_at_square(1), Piece::BlackKnight);
        assert_eq!(position.get_piece_at_square(2), Piece::BlackBishop);
        assert_eq!(position.get_piece_at_square(3), Piece::BlackQueen);
        assert_eq!(position.get_piece_at_square(4), Piece::BlackKing);
        assert_eq!(position.get_piece_at_square(5), Piece::BlackBishop);
        assert_eq!(position.get_piece_at_square(6), Piece::BlackKnight);
        assert_eq!(position.get_piece_at_square(7), Piece::BlackRook);
        assert_eq!(position.get_piece_at_square(8), Piece::BlackPawn);
        assert_eq!(position.get_piece_at_square(9), Piece::BlackPawn);
        assert_eq!(position.get_piece_at_square(10), Piece::BlackPawn);
        assert_eq!(position.get_piece_at_square(11), Piece::BlackPawn);
        assert_eq!(position.get_piece_at_square(12), Piece::BlackPawn);
        assert_eq!(position.get_piece_at_square(13), Piece::BlackPawn);
        assert_eq!(position.get_piece_at_square(14), Piece::BlackPawn);
        assert_eq!(position.get_piece_at_square(15), Piece::BlackPawn);
        assert_eq!(position.get_piece_at_square(48), Piece::WhitePawn);
        assert_eq!(position.get_piece_at_square(49), Piece::WhitePawn);
        assert_eq!(position.get_piece_at_square(50), Piece::WhitePawn);
        assert_eq!(position.get_piece_at_square(51), Piece::WhitePawn);
        assert_eq!(position.get_piece_at_square(52), Piece::WhitePawn);
        assert_eq!(position.get_piece_at_square(53), Piece::WhitePawn);
        assert_eq!(position.get_piece_at_square(54), Piece::WhitePawn);
        assert_eq!(position.get_piece_at_square(55), Piece::WhitePawn);
        assert_eq!(position.get_piece_at_square(56), Piece::WhiteRook);
        assert_eq!(position.get_piece_at_square(57), Piece::WhiteKnight);
    }

    #[ignore]
    #[test]
    fn generate_magic_numbers_correctly() {
        let mut board = Position::new(None);

        for square in SQUARE_ITER {
            let magic_number = board._find_magic_number(
                square,
                constants::ROOK_RELEVANT_BITS[usize::from(u8::from(square))],
                false,
            );

            assert_eq!(
                magic_number,
                constants::ROOK_MAGIC_NUMBERS[usize::from(u8::from(square))]
            );
        }

        for square in SQUARE_ITER {
            let magic_number = board._find_magic_number(
                square,
                constants::BISHOP_RELEVANT_BITS[usize::from(u8::from(square))],
                true,
            );

            assert_eq!(
                magic_number,
                constants::BISHOP_MAGIC_NUMBERS[usize::from(u8::from(square))]
            );
        }
    }

    #[bench]
    fn bench_get_piece_value(b: &mut Bencher) {
        let board = <Position as Board>::new(Some(STARTING_FEN));
        b.iter(|| {
            for square in SQUARE_ITER {
                let piece = board.get_piece_at_square(square as u8);

                if piece != Piece::Empty {
                    black_box(_get_piece_value_bl(
                        piece as usize,
                        square as usize,
                        board.get_game_phase_score(),
                    ));
                }
            }
        });
    }

    #[bench]
    fn bench_generate_moves(b: &mut Bencher) {
        let mut board = <Position as Board>::new(Some(
            "r3k2r/p1ppqpb1/Bn2pnpB/3PN3/1p2P3/2N2Q1p/PPP2PPP/R3K2R w KQkq - 0 1",
        ));
        b.iter(|| {
            board.generate_moves(false);
        })
    }

    #[bench]
    fn bench_generate_pawn_moves(b: &mut Bencher) {
        let board = <Position as Board>::new(Some(
            "r3k2r/p1ppqpb1/Bn2pnpB/3PN3/1p2P3/2N2Q1p/PPP2PPP/R3K2R w KQkq - 0 1",
        ));
        let mut moves = Vec::with_capacity(256);
        b.iter(|| {
            moves.clear();
            board.generate_pawn_moves(&mut moves, false);
        })
    }

    #[bench]
    fn bench_generate_knight_moves(b: &mut Bencher) {
        let board = <Position as Board>::new(Some(
            "r3k2r/p1ppqpb1/Bn2pnpB/3PN3/1p2P3/2N2Q1p/PPP2PPP/R3K2R w KQkq - 0 1",
        ));
        let mut moves = Vec::with_capacity(256);
        b.iter(|| {
            moves.clear();
            board.generate_knight_moves(&mut moves, false);
        })
    }

    #[bench]
    fn bench_generate_bishop_moves(b: &mut Bencher) {
        let board = <Position as Board>::new(Some(
            "r3k2r/p1ppqpb1/Bn2pnpB/3PN3/1p2P3/2N2Q1p/PPP2PPP/R3K2R w KQkq - 0 1",
        ));
        let mut moves = Vec::with_capacity(256);
        b.iter(|| {
            moves.clear();
            board.generate_bishop_moves(&mut moves, false);
        })
    }

    #[bench]
    fn bench_generate_rook_moves(b: &mut Bencher) {
        let board = <Position as Board>::new(Some(
            "r3k2r/p1ppqpb1/Bn2pnpB/3PN3/1p2P3/2N2Q1p/PPP2PPP/R3K2R w KQkq - 0 1",
        ));
        let mut moves = Vec::with_capacity(256);
        b.iter(|| {
            moves.clear();
            board.generate_rook_moves(&mut moves, false);
        })
    }

    #[bench]
    fn bench_generate_queen_moves(b: &mut Bencher) {
        let board = <Position as Board>::new(Some(
            "r3k2r/p1ppqpb1/Bn2pnpB/3PN3/1p2P3/2N2Q1p/PPP2PPP/R3K2R w KQkq - 0 1",
        ));
        let mut moves = Vec::with_capacity(256);
        b.iter(|| {
            moves.clear();
            board.generate_queen_moves(&mut moves, false);
        })
    }

    #[bench]
    fn bench_generate_king_moves(b: &mut Bencher) {
        let mut board = <Position as Board>::new(Some(
            "r3k2r/p1ppqpb1/Bn2pnpB/3PN3/1p2P3/2N2Q1p/PPP2PPP/R3K2R w KQkq - 0 1",
        ));
        let mut moves = Vec::with_capacity(256);
        b.iter(|| {
            moves.clear();
            board.generate_king_moves(&mut moves, false);
        })
    }
}

const OPENING_GAME_PHASE_SCORE: i32 = 6192;
const ENDGAME_PHASE_SCORE: i32 = 518;

pub const SIMPLE_PIECE_SCORES: [i32; 12] = [
    100, 320, 330, 500, 900, 20000, 100, 320, 330, 500, 900, 20000,
];

const OPENING_PIECE_SCORES: [i32; 12] = [
    82, 337, 365, 477, 1025, 12000, 82, 337, 365, 477, 1025, 12000,
];

const ENDING_PIECE_SCORES: [i32; 12] = [
    82, 337, 365, 477, 1025, 12000, 82, 337, 365, 477, 1025, 12000,
];

const OPENING_PIECE_PST_MAP: [[i32; 64]; 12] = [
    // White pieces
    OPEN_PAWN_POSITIONAL_SCORE,
    OPEN_KNIGHT_POSITIONAL_SCORE,
    OPEN_BISHOP_POSITIONAL_SCORE,
    OPEN_ROOK_POSITIONAL_SCORE,
    OPEN_QUEEN_POSITIONAL_SCORE,
    OPEN_KING_POSITIONAL_SCORE,
    // Black pieces
    OPEN_PAWN_POSITIONAL_SCORE,
    OPEN_KNIGHT_POSITIONAL_SCORE,
    OPEN_BISHOP_POSITIONAL_SCORE,
    OPEN_ROOK_POSITIONAL_SCORE,
    OPEN_QUEEN_POSITIONAL_SCORE,
    OPEN_KING_POSITIONAL_SCORE,
];

const ENDING_PIECE_PST_MAP: [[i32; 64]; 12] = [
    // White pieces
    END_PAWN_POSITIONAL_SCORE,
    END_KNIGHT_POSITIONAL_SCORE,
    END_BISHOP_POSITIONAL_SCORE,
    END_ROOK_POSITIONAL_SCORE,
    END_QUEEN_POSITIONAL_SCORE,
    END_KING_POSITIONAL_SCORE,
    // Black pieces
    END_PAWN_POSITIONAL_SCORE,
    END_KNIGHT_POSITIONAL_SCORE,
    END_BISHOP_POSITIONAL_SCORE,
    END_ROOK_POSITIONAL_SCORE,
    END_QUEEN_POSITIONAL_SCORE,
    END_KING_POSITIONAL_SCORE,
];

/*
        Material score values used for tapered evaluation

            Pawn  Knight  Bishop  Rook  Queen  King
opening  -  82    337     365     477   1025   12000
endgame  -  94    281     297     512   936    12000
*/

pub fn _get_piece_value_bl(piece: usize, square: usize, game_phase_score: i32) -> i32 {
    let is_opening = (game_phase_score > OPENING_GAME_PHASE_SCORE) as usize;
    let is_endgame = (game_phase_score < ENDGAME_PHASE_SCORE) as usize;
    let is_middlegame = 1 - (is_opening | is_endgame);

    let mirror_offset = ((piece) >= (Piece::BlackPawn as usize)) as usize;
    let adjusted_square =
        square * (1 - mirror_offset) + MIRROR_SCORE[square] as usize * mirror_offset;

    let opening_value = OPENING_PIECE_SCORES[piece] + OPENING_PIECE_PST_MAP[piece][adjusted_square];

    let ending_value = ENDING_PIECE_SCORES[piece] + ENDING_PIECE_PST_MAP[piece][adjusted_square];

    let middlegame_value = (OPENING_PIECE_SCORES[piece] * game_phase_score
        + ENDING_PIECE_SCORES[piece] * (OPENING_GAME_PHASE_SCORE - game_phase_score))
        / OPENING_GAME_PHASE_SCORE
        + (OPENING_PIECE_PST_MAP[piece][adjusted_square] * game_phase_score
            + ENDING_PIECE_PST_MAP[piece][adjusted_square]
                * (OPENING_GAME_PHASE_SCORE - game_phase_score))
            / OPENING_GAME_PHASE_SCORE;

    let piece_value = is_opening as i32 * opening_value
        + is_endgame as i32 * ending_value
        + is_middlegame as i32 * middlegame_value;

    return piece_value;
}
//
// pub fn _get_piece_value_bl(piece: Piece, square: usize, game_phase_score: i32) -> i32 {
//     if (piece as usize) < (Piece::BlackPawn as usize) {
//         let mut piece_value = 0;
//
//         if game_phase_score > OPENING_GAME_PHASE_SCORE {
//             // opening
//             piece_value += OPENING_PIECE_SCORES[piece as usize]
//                 + OPENING_PIECE_PST_MAP[piece as usize][square];
//         } else if game_phase_score < ENDGAME_PHASE_SCORE {
//             // endgame
//             piece_value +=
//                 ENDING_PIECE_SCORES[piece as usize] + ENDING_PIECE_PST_MAP[piece as usize][square];
//         } else {
//             // middlegame
//             piece_value += (OPENING_PIECE_SCORES[piece as usize] * game_phase_score
//                 + ENDING_PIECE_SCORES[piece as usize]
//                     * (OPENING_GAME_PHASE_SCORE - game_phase_score))
//                 / OPENING_GAME_PHASE_SCORE;
//
//             piece_value += (OPENING_PIECE_PST_MAP[piece as usize][square] * game_phase_score
//                 + ENDING_PIECE_PST_MAP[piece as usize][square]
//                     * (OPENING_GAME_PHASE_SCORE - game_phase_score))
//                 / OPENING_GAME_PHASE_SCORE;
//         };
//
//         return piece_value;
//     } else {
//         let mut piece_value = 0;
//
//         if game_phase_score > OPENING_GAME_PHASE_SCORE {
//             // opening
//             piece_value += OPENING_PIECE_SCORES[piece as usize]
//                 + OPENING_PIECE_PST_MAP[piece as usize][MIRROR_SCORE[square] as usize];
//         } else if game_phase_score < ENDGAME_PHASE_SCORE {
//             // endgame
//             piece_value += ENDING_PIECE_SCORES[piece as usize]
//                 + ENDING_PIECE_PST_MAP[piece as usize][MIRROR_SCORE[square] as usize];
//         } else {
//             // middlegame
//             piece_value += (OPENING_PIECE_SCORES[piece as usize] * game_phase_score
//                 + ENDING_PIECE_SCORES[piece as usize]
//                     * (OPENING_GAME_PHASE_SCORE - game_phase_score))
//                 / OPENING_GAME_PHASE_SCORE;
//
//             piece_value += (OPENING_PIECE_PST_MAP[piece as usize][MIRROR_SCORE[square] as usize]
//                 * game_phase_score
//                 + ENDING_PIECE_PST_MAP[piece as usize][MIRROR_SCORE[square] as usize]
//                     * (OPENING_GAME_PHASE_SCORE - game_phase_score))
//                 / OPENING_GAME_PHASE_SCORE;
//         };
//
//         return piece_value;
//     }
// }
