pub mod constants;

use crate::{
    chess::{
        self, castling_rights::CastlingRights, color::Color, piece::Piece, square::Square,
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

/// A chess position
pub struct Position {
    pub bitboards: [u64; 12],
    pub turn: Color,
    pub enpassant: Option<Square>,
    pub castling: CastlingRights,

    pub halfmove_clock: u32,
    pub fullmove_number: u32,

    pub pawn_attacks: [[u64; 64]; 2],
    pub knight_attacks: [u64; 64],
    pub king_attacks: [u64; 64],

    pub bishop_attacks: [u64; 64],
    pub rook_attacks: [u64; 64],
    pub queen_attacks: [u64; 64],

    pub magic_bishop_masks: [u64; 64],
    pub magic_rook_masks: [u64; 64],

    pub magic_bishop_attacks: Vec<Vec<u64>>,
    pub magic_rook_attacks: Vec<Vec<u64>>,

    pub position_stack: Vec<HistoryEntry>,

    pub rand_seed: u32,

    pub occupancies: [u64; 3],

    pub zobrist_piece_keys: [[u64; 64]; 12],
    pub zobrist_castling_keys: [u64; 16],
    pub zobrist_enpassant_keys: [u64; 64],
    pub zobrist_turn_key: u64,

    pub hash: u64,
    pub material: [i32; 2],

    pub file_masks: [u64; 64],
    pub rank_masks: [u64; 64],
    pub isolated_pawn_masks: [u64; 64],

    pub white_passed_pawn_masks: [u64; 64],
    pub black_passed_pawn_masks: [u64; 64],

    pub opponent_attack_map: u64,
}

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

    fn make_move(&mut self, m: chess::_move::BitPackedMove, only_captures: bool) -> bool;
    fn unmake_move(&mut self);
    fn make_null_move(&mut self);

    fn as_fen(&self) -> String;
}

impl Board for Position {
    fn make_null_move(&mut self) {
        // add the move to the history
        let history_entry = self.to_history_entry();
        self.position_stack.push(history_entry);

        self.turn = !self.turn;
        self.hash ^= self.zobrist_turn_key;
    }

    fn new(fen: Option<&str>) -> Position {
        let mut pos = Position {
            bitboards: [0; 12],
            turn: Color::White,

            pawn_attacks: [[0; 64]; 2],
            knight_attacks: [0; 64],
            king_attacks: [0; 64],
            bishop_attacks: [0; 64],
            rook_attacks: [0; 64],
            queen_attacks: [0; 64],

            magic_bishop_masks: [0; 64],
            magic_rook_masks: [0; 64],

            magic_bishop_attacks: vec![vec![0; 512]; 64],
            magic_rook_attacks: vec![vec![0; 4096]; 64],

            enpassant: None,
            castling: CastlingRights::new(),
            halfmove_clock: 0,
            fullmove_number: 1,

            rand_seed: 1804289383,

            position_stack: Vec::with_capacity(512),

            occupancies: [0; 3],

            zobrist_piece_keys: [[0; 64]; 12],
            zobrist_castling_keys: [0; 16],
            zobrist_enpassant_keys: [0; 64],
            zobrist_turn_key: 0,

            hash: 0,
            material: [0, 0],

            white_passed_pawn_masks: [0; 64],
            black_passed_pawn_masks: [0; 64],

            file_masks: [0; 64],
            rank_masks: [0; 64],
            isolated_pawn_masks: [0; 64],

            opponent_attack_map: 0,
        };

        pos.initialize_leaper_piece_attacks();
        pos.initialize_slider_piece_attacks();
        pos.initialize_slider_magic_attacks(false);
        pos.initialize_slider_magic_attacks(true);
        pos.init_evaluation_masks();

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
            let v = _get_piece_value(piece, i as usize, self.get_game_phase_score());
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

        let p_attacks = self.pawn_attacks[!color as usize][square as usize]
            & self.bitboards[Piece::WhitePawn as usize + piece_offset];

        let n_attacks = self.knight_attacks[square as usize]
            & self.bitboards[Piece::WhiteKnight as usize + piece_offset];

        let bq_attacks = self.get_bishop_magic_attacks(square, both_occupancy)
            & (self.bitboards[Piece::WhiteBishop as usize + piece_offset]
                | self.bitboards[Piece::WhiteQueen as usize + piece_offset]);

        let rq_attacks = self.get_rook_magic_attacks(square, both_occupancy)
            & (self.bitboards[Piece::WhiteRook as usize + piece_offset]
                | self.bitboards[Piece::WhiteQueen as usize + piece_offset]);

        let k_attacks = self.king_attacks[square as usize]
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

    fn make_move(&mut self, m: chess::_move::BitPackedMove, only_captures: bool) -> bool {
        if only_captures && !m.is_capture() {
            return false;
        }

        let game_phase_score = self.get_game_phase_score();

        // add the move to the history
        let history_entry = self.to_history_entry();
        self.position_stack.push(history_entry);

        // set the moving piece
        utils::set_bit(
            &mut self.bitboards[m.get_piece() as usize],
            m.get_to() as u8,
        );
        self.material[self.turn as usize] +=
            _get_piece_value(m.get_piece(), m.get_to() as usize, game_phase_score);

        self.hash ^= self.zobrist_piece_keys[m.get_piece() as usize][m.get_from() as usize];
        self.hash ^= self.zobrist_piece_keys[m.get_piece() as usize][m.get_to() as usize];

        // remove the moving piece
        utils::clear_bit(
            &mut self.bitboards[m.get_piece() as usize],
            m.get_from() as u8,
        );
        self.material[self.turn as usize] -=
            _get_piece_value(m.get_piece(), m.get_from() as usize, game_phase_score);

        // handle captures
        if m.is_capture() {
            let captured_piece = m.get_capture();
            // remove the captured piece
            utils::clear_bit(
                &mut self.bitboards[captured_piece as usize],
                m.get_to() as u8,
            );
            self.material[!self.turn as usize] -=
                _get_piece_value(captured_piece, m.get_to() as usize, game_phase_score);
            self.hash ^= self.zobrist_piece_keys[captured_piece as usize][m.get_to() as usize];
        }

        // handle promotions
        if m.is_promotion() {
            // remove the pawn
            utils::clear_bit(
                &mut self.bitboards[m.get_piece() as usize],
                m.get_to() as u8,
            );
            self.material[self.turn as usize] -=
                _get_piece_value(m.get_piece(), m.get_to() as usize, game_phase_score);
            self.hash ^= self.zobrist_piece_keys[m.get_piece() as usize][m.get_to() as usize];

            // add the promoted piece
            utils::set_bit(
                &mut self.bitboards[m.get_promotion() as usize],
                m.get_to() as u8,
            );
            self.material[self.turn as usize] +=
                _get_piece_value(m.get_promotion(), m.get_to() as usize, game_phase_score);
            self.hash ^= self.zobrist_piece_keys[m.get_promotion() as usize][m.get_to() as usize];
        }

        if let Some(enpassant_square) = self.enpassant {
            self.hash ^= self.zobrist_enpassant_keys[enpassant_square as usize];
        }

        // handle en passant
        if m.is_enpassant() {
            let en_captured_square = (m.get_to() as u8) + (8 - (self.turn as u8 * 16));

            if get_bit(self.occupancies[2], en_captured_square) != 0 {
                let en_captured_piece = self.get_piece_at_square(en_captured_square);

                // remove the captured pawn
                utils::clear_bit(
                    &mut self.bitboards[en_captured_piece as usize],
                    en_captured_square,
                );

                self.material[!self.turn as usize] -= _get_piece_value(
                    en_captured_piece,
                    en_captured_square as usize,
                    game_phase_score,
                );
            }
        }

        self.enpassant = None;

        // handle setting the en passant square during double pawn pushes
        if m.get_piece() == Piece::WhitePawn || m.get_piece() == Piece::BlackPawn {
            let offset = -1 * (m.get_from() as i8 - m.get_to() as i8) / 2;
            if offset == 8 || offset == -8 {
                self.enpassant = Some(Square::from((m.get_from() as i8 + offset) as u8));
                self.hash ^= self.zobrist_enpassant_keys[(m.get_from() as i8 + offset) as usize];
            }
        }

        // handle castling
        if m.is_castle() {
            match m.get_to() {
                Square::C1 => {
                    // white queen side
                    utils::clear_bit(
                        &mut self.bitboards[Piece::WhiteRook as usize],
                        Square::A1 as u8,
                    );
                    self.material[self.turn as usize] -=
                        _get_piece_value(Piece::WhiteRook, Square::A1 as usize, game_phase_score);
                    self.hash ^=
                        self.zobrist_piece_keys[Piece::WhiteRook as usize][Square::A1 as usize];

                    utils::set_bit(
                        &mut self.bitboards[Piece::WhiteRook as usize],
                        Square::D1 as u8,
                    );
                    self.material[self.turn as usize] +=
                        _get_piece_value(Piece::WhiteRook, Square::D1 as usize, game_phase_score);
                    self.hash ^=
                        self.zobrist_piece_keys[Piece::WhiteRook as usize][Square::D1 as usize];
                }
                Square::G1 => {
                    // white king side
                    utils::clear_bit(
                        &mut self.bitboards[Piece::WhiteRook as usize],
                        Square::H1 as u8,
                    );
                    self.material[self.turn as usize] -=
                        _get_piece_value(Piece::WhiteRook, Square::H1 as usize, game_phase_score);
                    self.hash ^=
                        self.zobrist_piece_keys[Piece::WhiteRook as usize][Square::H1 as usize];

                    utils::set_bit(
                        &mut self.bitboards[Piece::WhiteRook as usize],
                        Square::F1 as u8,
                    );
                    self.material[self.turn as usize] +=
                        _get_piece_value(Piece::WhiteRook, Square::F1 as usize, game_phase_score);
                    self.hash ^=
                        self.zobrist_piece_keys[Piece::WhiteRook as usize][Square::F1 as usize];
                }
                Square::C8 => {
                    // black queen side
                    utils::clear_bit(
                        &mut self.bitboards[Piece::BlackRook as usize],
                        Square::A8 as u8,
                    );
                    self.material[self.turn as usize] -=
                        _get_piece_value(Piece::BlackRook, Square::A8 as usize, game_phase_score);
                    self.hash ^=
                        self.zobrist_piece_keys[Piece::BlackRook as usize][Square::A8 as usize];

                    utils::set_bit(
                        &mut self.bitboards[Piece::BlackRook as usize],
                        Square::D8 as u8,
                    );
                    self.material[self.turn as usize] +=
                        _get_piece_value(Piece::BlackRook, Square::D8 as usize, game_phase_score);
                    self.hash ^=
                        self.zobrist_piece_keys[Piece::BlackRook as usize][Square::D8 as usize];
                }
                Square::G8 => {
                    // black king side
                    utils::clear_bit(
                        &mut self.bitboards[Piece::BlackRook as usize],
                        Square::H8 as u8,
                    );
                    self.material[self.turn as usize] -=
                        _get_piece_value(Piece::BlackRook, Square::H8 as usize, game_phase_score);
                    self.hash ^=
                        self.zobrist_piece_keys[Piece::BlackRook as usize][Square::H8 as usize];

                    utils::set_bit(
                        &mut self.bitboards[Piece::BlackRook as usize],
                        Square::F8 as u8,
                    );
                    self.material[self.turn as usize] +=
                        _get_piece_value(Piece::BlackRook, Square::F8 as usize, game_phase_score);
                    self.hash ^=
                        self.zobrist_piece_keys[Piece::BlackRook as usize][Square::F8 as usize];
                }
                _ => {}
            }
        }

        self.hash ^= self.zobrist_castling_keys[self.castling.get_rights_u8() as usize];

        if m.get_piece() == Piece::WhiteKing {
            self.castling
                .remove_right(CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE);
        } else if m.get_piece() == Piece::BlackKing {
            self.castling
                .remove_right(CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE);
        }

        match m.get_piece() {
            Piece::WhiteRook => {
                if m.get_from() == Square::A1 {
                    self.castling.remove_right(CastlingRights::WHITE_QUEENSIDE);
                } else if m.get_from() == Square::H1 {
                    self.castling.remove_right(CastlingRights::WHITE_KINGSIDE);
                }
            }
            Piece::BlackRook => {
                if m.get_from() == Square::A8 {
                    self.castling.remove_right(CastlingRights::BLACK_QUEENSIDE);
                } else if m.get_from() == Square::H8 {
                    self.castling.remove_right(CastlingRights::BLACK_KINGSIDE);
                }
            }
            _ => {}
        }

        self.hash ^= self.zobrist_castling_keys[self.castling.get_rights_u8() as usize];

        self.update_occupancies();

        // ensure the king is not in check
        if m.get_piece() != Piece::WhiteKing && m.get_piece() != Piece::BlackKing && !m.is_castle()
        {
            if self.is_in_check() {
                self.unmake_move();
                return false;
            }
        }

        self.turn = !self.turn;
        self.hash ^= self.zobrist_turn_key;

        return true;
    }

    fn unmake_move(&mut self) {
        // pop the history entry and apply it
        let history_entry = self.position_stack.pop().unwrap();
        self.apply_history_entry(history_entry);
    }
}

impl Position {
    pub fn set_file_rank_mask(&mut self, file_num: i32, rank_num: i32) -> u64 {
        let mut mask: u64 = 0;

        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;

                if file_num != -1 {
                    if file == file_num {
                        utils::set_bit(&mut mask, square as u8);
                    }
                } else if rank_num != -1 {
                    if rank == rank_num {
                        utils::set_bit(&mut mask, square as u8);
                    }
                }
            }
        }

        return mask;
    }

    pub fn init_evaluation_masks(&mut self) {
        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;

                self.file_masks[square] = self.set_file_rank_mask(file as i32, -1);
                self.rank_masks[square] = self.set_file_rank_mask(-1, rank as i32);

                self.isolated_pawn_masks[square] = self.set_file_rank_mask(file as i32 - 1, -1)
                    | self.set_file_rank_mask(file as i32 + 1, -1);
            }
        }

        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;

                let m = self.set_file_rank_mask(file as i32 - 1, -1)
                    | self.set_file_rank_mask(file as i32 + 1, -1)
                    | self.set_file_rank_mask(file as i32, -1);

                self.white_passed_pawn_masks[square] = m;
                self.black_passed_pawn_masks[square] = m;

                for i in 0..(8 - rank) {
                    self.white_passed_pawn_masks[square] &= !self.rank_masks[(7 - i) * 8 + file];
                }

                for i in 0..(rank + 1) {
                    self.black_passed_pawn_masks[square] &= !self.rank_masks[i * 8 + file];
                }
            }
        }
    }

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
        self.occupancies[Color::White as usize] = self.bitboards[Piece::WhitePawn as usize]
            | self.bitboards[Piece::WhiteKnight as usize]
            | self.bitboards[Piece::WhiteBishop as usize]
            | self.bitboards[Piece::WhiteRook as usize]
            | self.bitboards[Piece::WhiteQueen as usize]
            | self.bitboards[Piece::WhiteKing as usize];

        self.occupancies[Color::Black as usize] = self.bitboards[Piece::BlackPawn as usize]
            | self.bitboards[Piece::BlackKnight as usize]
            | self.bitboards[Piece::BlackBishop as usize]
            | self.bitboards[Piece::BlackRook as usize]
            | self.bitboards[Piece::BlackQueen as usize]
            | self.bitboards[Piece::BlackKing as usize];

        self.occupancies[2] =
            self.occupancies[Color::White as usize] | self.occupancies[Color::Black as usize];
    }

    pub fn apply_history_entry(&mut self, entry: HistoryEntry) {
        self.bitboards = entry.bitboards;
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
        for piece in chess::piece::PIECE_ITER {
            if utils::get_bit(self.bitboards[piece as usize], square) != 0 {
                return piece;
            }
        }
        return Piece::Empty;
    }

    fn initialize_leaper_piece_attacks(&mut self) {
        for i in 0..64 {
            // Pawns
            self.pawn_attacks[Color::White as usize][i] =
                self.mask_pawn_attacks(Square::from(i), Color::White);
            self.pawn_attacks[Color::Black as usize][i] =
                self.mask_pawn_attacks(Square::from(i), Color::Black);

            // Knights
            self.knight_attacks[i] = self.mask_knight_attacks(Square::from(i));

            // Kings
            self.king_attacks[i] = self.mask_king_attacks(Square::from(i));
        }
    }

    fn initialize_slider_piece_attacks(&mut self) {
        for i in 0..64 {
            // Bishops
            self.bishop_attacks[i] = self.mask_bishop_attacks(Square::from(i));

            // Rooks
            self.rook_attacks[i] = self.mask_rook_attacks(Square::from(i));

            // Queens
            self.queen_attacks[i] = self.rook_attacks[i] | self.bishop_attacks[i];
        }
    }

    fn mask_pawn_attacks(&self, square: Square, side: Color) -> u64 {
        let mut attacks: u64 = 0;
        let mut bitboard: u64 = 0;

        utils::set_bit(&mut bitboard, u8::from(square));

        if side == Color::White {
            if (bitboard >> 7) & !(*chess::constants::FILE_A) != 0 {
                attacks |= bitboard >> 7;
            }
            if (bitboard >> 9) & !(*chess::constants::FILE_H) != 0 {
                attacks |= bitboard >> 9;
            }
        }

        if side == Color::Black {
            if (bitboard << 7) & !(*chess::constants::FILE_H) != 0 {
                attacks |= bitboard << 7;
            }
            if (bitboard << 9) & !(*chess::constants::FILE_A) != 0 {
                attacks |= bitboard << 9;
            }
        }

        return attacks;
    }

    fn mask_knight_attacks(&self, square: Square) -> u64 {
        let mut attacks: u64 = 0;
        let mut bitboard: u64 = 0;

        utils::set_bit(&mut bitboard, u8::from(square));

        if (bitboard >> 17) & !(*chess::constants::FILE_H) != 0 {
            attacks |= bitboard >> 17;
        }
        if (bitboard >> 15) & !(*chess::constants::FILE_A) != 0 {
            attacks |= bitboard >> 15;
        }
        if (bitboard >> 10) & !(*chess::constants::FILE_GH) != 0 {
            attacks |= bitboard >> 10;
        }
        if (bitboard >> 6) & !(*chess::constants::FILE_AB) != 0 {
            attacks |= bitboard >> 6;
        }
        if (bitboard << 17) & !(*chess::constants::FILE_A) != 0 {
            attacks |= bitboard << 17;
        }
        if (bitboard << 15) & !(*chess::constants::FILE_H) != 0 {
            attacks |= bitboard << 15;
        }
        if (bitboard << 10) & !(*chess::constants::FILE_AB) != 0 {
            attacks |= bitboard << 10;
        }
        if (bitboard << 6) & !(*chess::constants::FILE_GH) != 0 {
            attacks |= bitboard << 6;
        }

        return attacks;
    }

    fn mask_king_attacks(&self, square: Square) -> u64 {
        let mut attacks: u64 = 0;
        let mut bitboard: u64 = 0;

        utils::set_bit(&mut bitboard, u8::from(square));

        if (bitboard >> 9) & !(*chess::constants::FILE_H) != 0 {
            attacks |= bitboard >> 9;
        }
        if (bitboard >> 8) != 0 {
            attacks |= bitboard >> 8;
        }
        if (bitboard >> 7) & !(*chess::constants::FILE_A) != 0 {
            attacks |= bitboard >> 7;
        }
        if (bitboard >> 1) & !(*chess::constants::FILE_H) != 0 {
            attacks |= bitboard >> 1;
        }

        if (bitboard << 9) & !(*chess::constants::FILE_A) != 0 {
            attacks |= bitboard << 9;
        }
        if (bitboard << 8) != 0 {
            attacks |= bitboard << 8;
        }
        if (bitboard << 7) & !(*chess::constants::FILE_H) != 0 {
            attacks |= bitboard << 7;
        }
        if (bitboard << 1) & !(*chess::constants::FILE_A) != 0 {
            attacks |= bitboard << 1;
        }

        return attacks;
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

    fn initialize_slider_magic_attacks(&mut self, is_bishop: bool) {
        for square in SQUARE_ITER {
            self.magic_bishop_masks[usize::from(u8::from(square))] =
                self.mask_bishop_attacks(square);

            self.magic_rook_masks[usize::from(u8::from(square))] = self.mask_rook_attacks(square);

            let attack_mask = if is_bishop {
                self.magic_bishop_masks[usize::from(u8::from(square))]
            } else {
                self.magic_rook_masks[usize::from(u8::from(square))]
            };

            let relevant_bits_count = utils::count_bits(attack_mask);

            let occupancy_indices = 1 << relevant_bits_count;

            for index in 0..occupancy_indices {
                if is_bishop {
                    let occupancy = self.set_occupancy(
                        index.try_into().unwrap(),
                        relevant_bits_count,
                        attack_mask,
                    );
                    let magic_index = ((occupancy.wrapping_mul(
                        constants::BISHOP_MAGIC_NUMBERS[usize::from(u8::from(square))],
                    )) >> (64
                        - constants::BISHOP_RELEVANT_BITS[usize::from(u8::from(square))]))
                        as usize;

                    self.magic_bishop_attacks[usize::from(u8::from(square))][magic_index] =
                        self.generate_bishop_attacks_on_the_fly(square, occupancy);
                } else {
                    let occupancy = self.set_occupancy(
                        index.try_into().unwrap(),
                        relevant_bits_count,
                        attack_mask,
                    );
                    let magic_index = ((occupancy.wrapping_mul(
                        constants::ROOK_MAGIC_NUMBERS[usize::from(u8::from(square))],
                    )) >> (64
                        - constants::ROOK_RELEVANT_BITS[usize::from(u8::from(square))]))
                        as usize;

                    self.magic_rook_attacks[usize::from(u8::from(square))][magic_index] =
                        self.generate_rook_attacks_on_the_fly(square, occupancy);
                }
            }
        }
    }

    pub fn get_bishop_magic_attacks(&self, square: Square, occupancy: u64) -> u64 {
        let mut mutable_occupancy = occupancy;
        mutable_occupancy &= self.magic_bishop_masks[usize::from(u8::from(square))];
        mutable_occupancy = mutable_occupancy
            .wrapping_mul(constants::BISHOP_MAGIC_NUMBERS[usize::from(u8::from(square))]);
        mutable_occupancy >>= 64 - constants::BISHOP_RELEVANT_BITS[usize::from(u8::from(square))];
        return self.magic_bishop_attacks[usize::from(u8::from(square))]
            [mutable_occupancy as usize];
    }

    pub fn get_rook_magic_attacks(&self, square: Square, occupancy: u64) -> u64 {
        let mut mutable_occupancy = occupancy;
        mutable_occupancy &= self.magic_rook_masks[usize::from(u8::from(square))];
        mutable_occupancy = mutable_occupancy
            .wrapping_mul(constants::ROOK_MAGIC_NUMBERS[usize::from(u8::from(square))]);
        mutable_occupancy >>= 64 - constants::ROOK_RELEVANT_BITS[usize::from(u8::from(square))];
        return self.magic_rook_attacks[usize::from(u8::from(square))][mutable_occupancy as usize];
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
        chess::{piece::Piece, square::SQUARE_ITER},
    };

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
}

const OPENING_GAME_PHASE_SCORE: i32 = 6192;
const ENDGAME_PHASE_SCORE: i32 = 518;

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
pub fn _get_piece_value(piece: Piece, square: usize, game_phase_score: i32) -> i32 {
    if (piece as usize) < (Piece::BlackPawn as usize) {
        let mut piece_value = 0;

        if game_phase_score > OPENING_GAME_PHASE_SCORE {
            // opening
            piece_value += OPENING_PIECE_SCORES[piece as usize]
                + OPENING_PIECE_PST_MAP[piece as usize][square];
        } else if game_phase_score < ENDGAME_PHASE_SCORE {
            // endgame
            piece_value +=
                ENDING_PIECE_SCORES[piece as usize] + ENDING_PIECE_PST_MAP[piece as usize][square];
        } else {
            // middlegame
            piece_value += (OPENING_PIECE_SCORES[piece as usize] * game_phase_score
                + ENDING_PIECE_SCORES[piece as usize]
                    * (OPENING_GAME_PHASE_SCORE - game_phase_score))
                / OPENING_GAME_PHASE_SCORE;

            piece_value += (OPENING_PIECE_PST_MAP[piece as usize][square] * game_phase_score
                + ENDING_PIECE_PST_MAP[piece as usize][square]
                    * (OPENING_GAME_PHASE_SCORE - game_phase_score))
                / OPENING_GAME_PHASE_SCORE;
        };

        return piece_value;
    } else {
        let mut piece_value = 0;

        if game_phase_score > OPENING_GAME_PHASE_SCORE {
            // opening
            piece_value += OPENING_PIECE_SCORES[piece as usize]
                + OPENING_PIECE_PST_MAP[piece as usize][MIRROR_SCORE[square] as usize];
        } else if game_phase_score < ENDGAME_PHASE_SCORE {
            // endgame
            piece_value += ENDING_PIECE_SCORES[piece as usize]
                + ENDING_PIECE_PST_MAP[piece as usize][MIRROR_SCORE[square] as usize];
        } else {
            // middlegame
            piece_value += (OPENING_PIECE_SCORES[piece as usize] * game_phase_score
                + ENDING_PIECE_SCORES[piece as usize]
                    * (OPENING_GAME_PHASE_SCORE - game_phase_score))
                / OPENING_GAME_PHASE_SCORE;

            piece_value += (OPENING_PIECE_PST_MAP[piece as usize][MIRROR_SCORE[square] as usize]
                * game_phase_score
                + ENDING_PIECE_PST_MAP[piece as usize][MIRROR_SCORE[square] as usize]
                    * (OPENING_GAME_PHASE_SCORE - game_phase_score))
                / OPENING_GAME_PHASE_SCORE;
        };

        return piece_value;
    }
}
