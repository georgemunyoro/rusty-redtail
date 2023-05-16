use crate::{
    chess, pst,
    utils::{self, pop_lsb},
};

mod constants {
    pub static BISHOP_RELEVANT_BITS: [u32; 64] = [
        6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7,
        5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5,
        5, 5, 5, 6,
    ];

    pub static ROOK_RELEVANT_BITS: [u32; 64] = [
        12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10,
        11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10,
        10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
    ];

    pub static BISHOP_MAGIC_NUMBERS: [u64; 64] = [
        0x40040844404084u64,
        0x2004208a004208u64,
        0x10190041080202u64,
        0x108060845042010u64,
        0x581104180800210u64,
        0x2112080446200010u64,
        0x1080820820060210u64,
        0x3c0808410220200u64,
        0x4050404440404u64,
        0x21001420088u64,
        0x24d0080801082102u64,
        0x1020a0a020400u64,
        0x40308200402u64,
        0x4011002100800u64,
        0x401484104104005u64,
        0x801010402020200u64,
        0x400210c3880100u64,
        0x404022024108200u64,
        0x810018200204102u64,
        0x4002801a02003u64,
        0x85040820080400u64,
        0x810102c808880400u64,
        0xe900410884800u64,
        0x8002020480840102u64,
        0x220200865090201u64,
        0x2010100a02021202u64,
        0x152048408022401u64,
        0x20080002081110u64,
        0x4001001021004000u64,
        0x800040400a011002u64,
        0xe4004081011002u64,
        0x1c004001012080u64,
        0x8004200962a00220u64,
        0x8422100208500202u64,
        0x2000402200300c08u64,
        0x8646020080080080u64,
        0x80020a0200100808u64,
        0x2010004880111000u64,
        0x623000a080011400u64,
        0x42008c0340209202u64,
        0x209188240001000u64,
        0x400408a884001800u64,
        0x110400a6080400u64,
        0x1840060a44020800u64,
        0x90080104000041u64,
        0x201011000808101u64,
        0x1a2208080504f080u64,
        0x8012020600211212u64,
        0x500861011240000u64,
        0x180806108200800u64,
        0x4000020e01040044u64,
        0x300000261044000au64,
        0x802241102020002u64,
        0x20906061210001u64,
        0x5a84841004010310u64,
        0x4010801011c04u64,
        0xa010109502200u64,
        0x4a02012000u64,
        0x500201010098b028u64,
        0x8040002811040900u64,
        0x28000010020204u64,
        0x6000020202d0240u64,
        0x8918844842082200u64,
        0x4010011029020020u64,
    ];

    pub static ROOK_MAGIC_NUMBERS: [u64; 64] = [
        0x8a80104000800020u64,
        0x140002000100040u64,
        0x2801880a0017001u64,
        0x100081001000420u64,
        0x200020010080420u64,
        0x3001c0002010008u64,
        0x8480008002000100u64,
        0x2080088004402900u64,
        0x800098204000u64,
        0x2024401000200040u64,
        0x100802000801000u64,
        0x120800800801000u64,
        0x208808088000400u64,
        0x2802200800400u64,
        0x2200800100020080u64,
        0x801000060821100u64,
        0x80044006422000u64,
        0x100808020004000u64,
        0x12108a0010204200u64,
        0x140848010000802u64,
        0x481828014002800u64,
        0x8094004002004100u64,
        0x4010040010010802u64,
        0x20008806104u64,
        0x100400080208000u64,
        0x2040002120081000u64,
        0x21200680100081u64,
        0x20100080080080u64,
        0x2000a00200410u64,
        0x20080800400u64,
        0x80088400100102u64,
        0x80004600042881u64,
        0x4040008040800020u64,
        0x440003000200801u64,
        0x4200011004500u64,
        0x188020010100100u64,
        0x14800401802800u64,
        0x2080040080800200u64,
        0x124080204001001u64,
        0x200046502000484u64,
        0x480400080088020u64,
        0x1000422010034000u64,
        0x30200100110040u64,
        0x100021010009u64,
        0x2002080100110004u64,
        0x202008004008002u64,
        0x20020004010100u64,
        0x2048440040820001u64,
        0x101002200408200u64,
        0x40802000401080u64,
        0x4008142004410100u64,
        0x2060820c0120200u64,
        0x1001004080100u64,
        0x20c020080040080u64,
        0x2935610830022400u64,
        0x44440041009200u64,
        0x280001040802101u64,
        0x2100190040002085u64,
        0x80c0084100102001u64,
        0x4024081001000421u64,
        0x20030a0244872u64,
        0x12001008414402u64,
        0x2006104900a0804u64,
        0x1004081002402u64,
    ];
}

/// A chess position
pub struct Position {
    pub bitboards: [u64; 12],
    pub turn: chess::Color,
    pub enpassant: Option<chess::Square>,
    pub castling: chess::CastlingRights,

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
}

pub struct HistoryEntry {
    pub bitboards: [u64; 12],
    pub turn: chess::Color,
    pub enpassant: Option<chess::Square>,
    pub castling: chess::CastlingRights,
    pub material: [i32; 2],

    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub occupancies: [u64; 3],
}

pub trait Board {
    fn new(fen: Option<&str>) -> Position;
    fn draw(&mut self);
    fn set_fen(&mut self, fen: String);
    fn is_square_attacked(&self, square: chess::Square, color: chess::Color) -> bool;
    fn get_game_phase_score(&self) -> u64;

    fn is_in_check(&self) -> bool;

    fn make_move(&mut self, m: chess::Move, only_captures: bool) -> bool;
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
        self.update_hash();
    }

    fn new(fen: Option<&str>) -> Position {
        let mut pos = Position {
            bitboards: [0; 12],
            turn: chess::Color::White,

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
            castling: chess::CastlingRights::new(),
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
        };

        pos.initialize_leaper_piece_attacks();
        pos.initialize_slider_piece_attacks();
        pos.initialize_slider_magic_attacks(false);
        pos.initialize_slider_magic_attacks(true);

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
        let king_square = if self.turn == chess::Color::White {
            utils::get_lsb(self.bitboards[chess::Piece::WhiteKing as usize])
        } else {
            utils::get_lsb(self.bitboards[chess::Piece::BlackKing as usize])
        };
        if king_square >= 64 {
            return true;
        }
        self.is_square_attacked(chess::Square::from(king_square), !self.turn)
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

            let piece = self.get_piece_at_square(i);
            if piece == chess::Piece::Empty {
                empty += 1;
            } else {
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
            chess::Color::White => "w",
            chess::Color::Black => "b",
        });

        // Castling availability
        fen.push(' ');
        if self
            .castling
            .can_castle(chess::CastlingRights::WHITE_KINGSIDE)
        {
            fen.push('K');
        }
        if self
            .castling
            .can_castle(chess::CastlingRights::WHITE_QUEENSIDE)
        {
            fen.push('Q');
        }
        if self
            .castling
            .can_castle(chess::CastlingRights::BLACK_KINGSIDE)
        {
            fen.push('k');
        }
        if self
            .castling
            .can_castle(chess::CastlingRights::BLACK_QUEENSIDE)
        {
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

            utils::set_bit(&mut self.bitboards[chess::Piece::from(c) as usize], pos);
            let piece_color = if c.is_uppercase() {
                chess::Color::White
            } else {
                chess::Color::Black
            };

            self.material[piece_color as usize] += _get_piece_value(
                chess::Piece::from(c),
                pos as usize,
                self.get_game_phase_score(),
            );

            pos += 1;
        }

        // Set the turn
        self.turn = chess::Color::from(sections[1].chars().next().unwrap());

        // Set the castling rights
        self.castling = chess::CastlingRights::from(sections[2]);

        // Set the en passant square
        self.enpassant = match sections[3] {
            "-" => None,
            _ => Some(chess::Square::from(sections[3])),
        };

        // Set the halfmove clock
        self.halfmove_clock = sections[4].parse::<u32>().unwrap();

        // Set the fullmove number
        self.fullmove_number = sections[5].parse::<u32>().unwrap();

        self.update_occupancies();
        self.update_hash();
    }

    /// Returns true if the given square is attacked by the given color
    fn is_square_attacked(&self, square: chess::Square, color: chess::Color) -> bool {
        if color == chess::Color::White {
            // attacked by white pawns
            if (self.pawn_attacks[chess::Color::Black as usize][square as usize]
                & self.bitboards[chess::Piece::WhitePawn as usize])
                != 0
            {
                return true;
            }

            // attackd by white bishops
            if self.get_bishop_magic_attacks(square, self.get_both_occupancy())
                & self.bitboards[chess::Piece::WhiteBishop as usize]
                != 0
            {
                return true;
            }
        } else {
            // attacked by black pawns
            if (self.pawn_attacks[chess::Color::White as usize][square as usize]
                & self.bitboards[chess::Piece::BlackPawn as usize])
                != 0
            {
                return true;
            }

            // attackd by black bishops
            if self.get_bishop_magic_attacks(square, self.get_both_occupancy())
                & self.bitboards[chess::Piece::BlackBishop as usize]
                != 0
            {
                return true;
            }
        }

        // attacked by knights
        let knight_piece = if color == chess::Color::White {
            chess::Piece::WhiteKnight
        } else {
            chess::Piece::BlackKnight
        };
        if (self.knight_attacks[square as usize] & self.bitboards[knight_piece as usize]) != 0 {
            return true;
        }

        // attacked by bishops
        let bishop_piece = if color == chess::Color::White {
            chess::Piece::WhiteBishop
        } else {
            chess::Piece::BlackBishop
        };
        if (self.get_bishop_magic_attacks(square, self.get_both_occupancy())
            & self.bitboards[bishop_piece as usize])
            != 0
        {
            return true;
        }

        // attacked by rooks
        let rook_piece = if color == chess::Color::White {
            chess::Piece::WhiteRook
        } else {
            chess::Piece::BlackRook
        };
        if (self.get_rook_magic_attacks(square, self.get_both_occupancy())
            & self.bitboards[rook_piece as usize])
            != 0
        {
            return true;
        }

        // attacked by queens
        let queen_piece = if color == chess::Color::White {
            chess::Piece::WhiteQueen
        } else {
            chess::Piece::BlackQueen
        };
        if (self.get_queen_magic_attacks(square, self.get_both_occupancy())
            & self.bitboards[queen_piece as usize])
            != 0
        {
            return true;
        }

        // attacked by kings
        let king_piece = if color == chess::Color::White {
            chess::Piece::WhiteKing
        } else {
            chess::Piece::BlackKing
        };
        if (self.king_attacks[square as usize] & self.bitboards[king_piece as usize]) != 0 {
            return true;
        }

        return false;
    }

    fn get_game_phase_score(&self) -> u64 {
        (4 * utils::count_bits(
            self.bitboards[chess::Piece::BlackBishop as usize]
                | self.bitboards[chess::Piece::WhiteBishop as usize],
        )) + (4 * utils::count_bits(
            self.bitboards[chess::Piece::BlackKnight as usize]
                | self.bitboards[chess::Piece::WhiteKnight as usize],
        )) + (4 * utils::count_bits(
            self.bitboards[chess::Piece::BlackRook as usize]
                | self.bitboards[chess::Piece::WhiteRook as usize],
        )) + (2 * utils::count_bits(
            self.bitboards[chess::Piece::WhiteQueen as usize]
                | self.bitboards[chess::Piece::BlackQueen as usize],
        ))
    }

    fn make_move(&mut self, m: chess::Move, only_captures: bool) -> bool {
        let is_capture = m.capture.is_some();

        if only_captures && !is_capture {
            return false;
        }

        let game_phase_score = self.get_game_phase_score();

        // add the move to the history
        let history_entry = self.to_history_entry();
        self.position_stack.push(history_entry);

        // set the moving piece
        utils::set_bit(&mut self.bitboards[m.piece as usize], m.to as u8);
        self.material[self.turn as usize] +=
            _get_piece_value(m.piece, m.to as usize, game_phase_score);

        // remove the moving piece
        utils::clear_bit(&mut self.bitboards[m.piece as usize], m.from as u8);
        self.material[self.turn as usize] -=
            _get_piece_value(m.piece, m.from as usize, game_phase_score);

        // handle captures
        if is_capture {
            let captured_piece = m.capture.unwrap();
            // remove the captured piece
            utils::clear_bit(&mut self.bitboards[captured_piece as usize], m.to as u8);
            self.material[!self.turn as usize] -=
                _get_piece_value(captured_piece, m.to as usize, game_phase_score);
        }

        // handle promotions
        match m.promotion {
            None => {}
            Some(promotion_piece) => {
                // remove the pawn
                utils::clear_bit(&mut self.bitboards[m.piece as usize], m.to as u8);
                self.material[self.turn as usize] -=
                    _get_piece_value(m.piece, m.to as usize, game_phase_score);
                // add the promoted piece
                utils::set_bit(&mut self.bitboards[promotion_piece as usize], m.to as u8);
                self.material[self.turn as usize] +=
                    _get_piece_value(promotion_piece, m.to as usize, game_phase_score);
            }
        }

        // handle en passant
        if m.en_passant {
            let en_captured_square = if self.turn == chess::Color::White {
                (m.to as u8) + 8
            } else {
                (m.to as u8) - 8
            };

            let en_captured_piece = self.get_piece_at_square(en_captured_square);

            if en_captured_piece != chess::Piece::Empty {
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
        if m.piece == chess::Piece::WhitePawn || m.piece == chess::Piece::BlackPawn {
            if m.from as i8 - m.to as i8 == 16 {
                self.enpassant = Some(chess::Square::from(m.from as u8 - 8));
            } else if m.from as i8 - m.to as i8 == -16 {
                self.enpassant = Some(chess::Square::from(m.from as u8 + 8));
            }
        }

        // handle castling
        if m.castle {
            match m.to {
                chess::Square::C1 => {
                    // white queen side
                    utils::clear_bit(
                        &mut self.bitboards[chess::Piece::WhiteRook as usize],
                        chess::Square::A1 as u8,
                    );
                    self.material[self.turn as usize] -= _get_piece_value(
                        chess::Piece::WhiteRook,
                        chess::Square::A1 as usize,
                        game_phase_score,
                    );

                    utils::set_bit(
                        &mut self.bitboards[chess::Piece::WhiteRook as usize],
                        chess::Square::D1 as u8,
                    );
                    self.material[self.turn as usize] += _get_piece_value(
                        chess::Piece::WhiteRook,
                        chess::Square::D1 as usize,
                        game_phase_score,
                    );
                }
                chess::Square::G1 => {
                    // white king side
                    utils::clear_bit(
                        &mut self.bitboards[chess::Piece::WhiteRook as usize],
                        chess::Square::H1 as u8,
                    );
                    self.material[self.turn as usize] -= _get_piece_value(
                        chess::Piece::WhiteRook,
                        chess::Square::H1 as usize,
                        game_phase_score,
                    );

                    utils::set_bit(
                        &mut self.bitboards[chess::Piece::WhiteRook as usize],
                        chess::Square::F1 as u8,
                    );
                    self.material[self.turn as usize] += _get_piece_value(
                        chess::Piece::WhiteRook,
                        chess::Square::F1 as usize,
                        game_phase_score,
                    );
                }
                chess::Square::C8 => {
                    // black queen side
                    utils::clear_bit(
                        &mut self.bitboards[chess::Piece::BlackRook as usize],
                        chess::Square::A8 as u8,
                    );
                    self.material[self.turn as usize] -= _get_piece_value(
                        chess::Piece::BlackRook,
                        chess::Square::A8 as usize,
                        game_phase_score,
                    );

                    utils::set_bit(
                        &mut self.bitboards[chess::Piece::BlackRook as usize],
                        chess::Square::D8 as u8,
                    );
                    self.material[self.turn as usize] += _get_piece_value(
                        chess::Piece::BlackRook,
                        chess::Square::D8 as usize,
                        game_phase_score,
                    );
                }
                chess::Square::G8 => {
                    // black king side
                    utils::clear_bit(
                        &mut self.bitboards[chess::Piece::BlackRook as usize],
                        chess::Square::H8 as u8,
                    );
                    self.material[self.turn as usize] -= _get_piece_value(
                        chess::Piece::BlackRook,
                        chess::Square::H8 as usize,
                        game_phase_score,
                    );

                    utils::set_bit(
                        &mut self.bitboards[chess::Piece::BlackRook as usize],
                        chess::Square::F8 as u8,
                    );
                    self.material[self.turn as usize] += _get_piece_value(
                        chess::Piece::BlackRook,
                        chess::Square::F8 as usize,
                        game_phase_score,
                    );
                }
                _ => {}
            }
        }

        if m.piece == chess::Piece::WhiteKing {
            self.castling.remove_right(
                chess::CastlingRights::WHITE_KINGSIDE | chess::CastlingRights::WHITE_QUEENSIDE,
            );
        } else if m.piece == chess::Piece::BlackKing {
            self.castling.remove_right(
                chess::CastlingRights::BLACK_KINGSIDE | chess::CastlingRights::BLACK_QUEENSIDE,
            );
        }

        // First move of a rook disables castling
        if m.piece == chess::Piece::WhiteRook
            && m.from == chess::Square::A1
            && self
                .castling
                .can_castle(chess::CastlingRights::WHITE_QUEENSIDE)
        {
            self.castling
                .remove_right(chess::CastlingRights::WHITE_QUEENSIDE);
        }

        if m.piece == chess::Piece::WhiteRook
            && m.from == chess::Square::H1
            && self
                .castling
                .can_castle(chess::CastlingRights::WHITE_KINGSIDE)
        {
            self.castling
                .remove_right(chess::CastlingRights::WHITE_KINGSIDE);
        }

        if m.piece == chess::Piece::BlackRook
            && m.from == chess::Square::A8
            && self
                .castling
                .can_castle(chess::CastlingRights::BLACK_QUEENSIDE)
        {
            self.castling
                .remove_right(chess::CastlingRights::BLACK_QUEENSIDE);
        }

        if m.piece == chess::Piece::BlackRook
            && m.from == chess::Square::H8
            && self
                .castling
                .can_castle(chess::CastlingRights::BLACK_KINGSIDE)
        {
            self.castling
                .remove_right(chess::CastlingRights::BLACK_KINGSIDE);
        }

        self.update_occupancies();

        // ensure the king is not in check
        if self.is_in_check() {
            self.unmake_move();
            return false;
        }

        self.turn = !self.turn;
        self.update_hash();

        return true;
    }

    fn unmake_move(&mut self) {
        // pop the history entry and apply it
        let history_entry = self.position_stack.pop().unwrap();
        self.apply_history_entry(history_entry);

        self.update_hash();
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

        if self.turn == chess::Color::Black {
            hash ^= self.zobrist_turn_key;
        }

        self.hash = hash;
    }

    pub fn init_zorbrist_keys(&mut self) {
        self.rand_seed = 1804289383;

        for piece in (chess::Piece::WhitePawn as usize)..=(chess::Piece::BlackKing as usize) {
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
        };
    }

    /// updates the occupancies bitboards
    fn update_occupancies(&mut self) {
        self.occupancies[chess::Color::Black as usize] = self.get_occupancy(chess::Color::Black);
        self.occupancies[chess::Color::White as usize] = self.get_occupancy(chess::Color::White);
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
    }

    pub fn get_piece_at_square(&self, square: u8) -> chess::Piece {
        for piece in chess::PIECE_ITER {
            if utils::get_bit(self.bitboards[piece as usize], square) != 0 {
                return piece;
            }
        }
        return chess::Piece::Empty;
    }

    fn initialize_leaper_piece_attacks(&mut self) {
        for i in 0..64 {
            // Pawns
            self.pawn_attacks[chess::Color::White as usize][i] =
                self.mask_pawn_attacks(chess::Square::from(i), chess::Color::White);
            self.pawn_attacks[chess::Color::Black as usize][i] =
                self.mask_pawn_attacks(chess::Square::from(i), chess::Color::Black);

            // Knights
            self.knight_attacks[i] = self.mask_knight_attacks(chess::Square::from(i));

            // Kings
            self.king_attacks[i] = self.mask_king_attacks(chess::Square::from(i));
        }
    }

    fn initialize_slider_piece_attacks(&mut self) {
        for i in 0..64 {
            // Bishops
            self.bishop_attacks[i] = self.mask_bishop_attacks(chess::Square::from(i));

            // Rooks
            self.rook_attacks[i] = self.mask_rook_attacks(chess::Square::from(i));

            // Queens
            self.queen_attacks[i] = self.rook_attacks[i] | self.bishop_attacks[i];
        }
    }

    fn mask_pawn_attacks(&self, square: chess::Square, side: chess::Color) -> u64 {
        let mut attacks: u64 = 0;
        let mut bitboard: u64 = 0;

        utils::set_bit(&mut bitboard, u8::from(square));

        if side == chess::Color::White {
            if (bitboard >> 7) & !(*chess::constants::FILE_A) != 0 {
                attacks |= bitboard >> 7;
            }
            if (bitboard >> 9) & !(*chess::constants::FILE_H) != 0 {
                attacks |= bitboard >> 9;
            }
        }

        if side == chess::Color::Black {
            if (bitboard << 7) & !(*chess::constants::FILE_H) != 0 {
                attacks |= bitboard << 7;
            }
            if (bitboard << 9) & !(*chess::constants::FILE_A) != 0 {
                attacks |= bitboard << 9;
            }
        }

        return attacks;
    }

    fn mask_knight_attacks(&self, square: chess::Square) -> u64 {
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

    fn mask_king_attacks(&self, square: chess::Square) -> u64 {
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

    fn mask_bishop_attacks(&self, square: chess::Square) -> u64 {
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

    fn mask_rook_attacks(&self, square: chess::Square) -> u64 {
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

    fn generate_bishop_attacks_on_the_fly(&self, square: chess::Square, blockers: u64) -> u64 {
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

    fn generate_rook_attacks_on_the_fly(&self, square: chess::Square, blockers: u64) -> u64 {
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

    fn _find_magic_number(
        &mut self,
        square: chess::Square,
        relevant_bits: u32,
        is_bishop: bool,
    ) -> u64 {
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

        for square in chess::SQUARE_ITER {
            let magic_number = self._find_magic_number(
                square,
                constants::ROOK_RELEVANT_BITS[usize::from(u8::from(square))],
                false,
            );

            println!("Bishop {} 0x{:016x}u64", square, magic_number);
        }

        println!("--------------------------------");

        for square in chess::SQUARE_ITER {
            let magic_number = self._find_magic_number(
                square,
                constants::BISHOP_RELEVANT_BITS[usize::from(u8::from(square))],
                true,
            );

            println!("Rook {} 0x{:016x}u64", square, magic_number);
        }
    }

    fn initialize_slider_magic_attacks(&mut self, is_bishop: bool) {
        for square in chess::SQUARE_ITER {
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

    pub fn get_bishop_magic_attacks(&self, square: chess::Square, occupancy: u64) -> u64 {
        let mut mutable_occupancy = occupancy;
        mutable_occupancy &= self.magic_bishop_masks[usize::from(u8::from(square))];
        mutable_occupancy = mutable_occupancy
            .wrapping_mul(constants::BISHOP_MAGIC_NUMBERS[usize::from(u8::from(square))]);
        mutable_occupancy >>= 64 - constants::BISHOP_RELEVANT_BITS[usize::from(u8::from(square))];
        return self.magic_bishop_attacks[usize::from(u8::from(square))]
            [mutable_occupancy as usize];
    }

    pub fn get_rook_magic_attacks(&self, square: chess::Square, occupancy: u64) -> u64 {
        let mut mutable_occupancy = occupancy;
        mutable_occupancy &= self.magic_rook_masks[usize::from(u8::from(square))];
        mutable_occupancy = mutable_occupancy
            .wrapping_mul(constants::ROOK_MAGIC_NUMBERS[usize::from(u8::from(square))]);
        mutable_occupancy >>= 64 - constants::ROOK_RELEVANT_BITS[usize::from(u8::from(square))];
        return self.magic_rook_attacks[usize::from(u8::from(square))][mutable_occupancy as usize];
    }

    pub fn get_queen_magic_attacks(&self, square: chess::Square, occupancy: u64) -> u64 {
        return self.get_bishop_magic_attacks(square, occupancy)
            | self.get_rook_magic_attacks(square, occupancy);
    }

    pub fn get_occupancy(&self, color: chess::Color) -> u64 {
        let mut white_occupancy = 0u64;
        let mut black_occupancy = 0u64;

        for piece in (chess::Piece::WhitePawn as u8)..=(chess::Piece::WhiteKing as u8) {
            white_occupancy |= self.bitboards[piece as usize];
        }

        for piece in (chess::Piece::BlackPawn as u8)..=(chess::Piece::BlackKing as u8) {
            black_occupancy |= self.bitboards[piece as usize];
        }

        if color == chess::Color::White {
            return white_occupancy;
        }

        if color == chess::Color::Black {
            return black_occupancy;
        }

        return white_occupancy | black_occupancy;
    }

    pub fn get_both_occupancy(&self) -> u64 {
        let mut white_occupancy = 0u64;
        let mut black_occupancy = 0u64;

        for piece in (chess::Piece::WhitePawn as u8)..=(chess::Piece::WhiteKing as u8) {
            white_occupancy |= self.bitboards[piece as usize];
        }

        for piece in (chess::Piece::BlackPawn as u8)..=(chess::Piece::BlackKing as u8) {
            black_occupancy |= self.bitboards[piece as usize];
        }

        return white_occupancy | black_occupancy;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{constants, Board, Position},
        chess::{self, Piece},
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

        for square in chess::SQUARE_ITER {
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

        for square in chess::SQUARE_ITER {
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

/*
        Material score values used for tapered evaluation

            Pawn  Knight  Bishop  Rook  Queen  King
opening  -  82    337     365     477   1025   12000
endgame  -  94    281     297     512   936    12000
*/
pub fn _get_piece_value(piece: chess::Piece, square: usize, phase: u64) -> i32 {
    if (piece as usize) < (chess::Piece::BlackPawn as usize) {
        match piece {
            chess::Piece::WhitePawn => {
                return pst::PAWN_POSITIONAL_SCORE[square]
                    + (((82 * (256 - phase as i32)) + (94 * phase as i32)) / 256);
            }
            chess::Piece::WhiteKnight => {
                return pst::KNIGHT_POSITIONAL_SCORE[square]
                    + (((337 * (256 - phase as i32)) + (281 * phase as i32)) / 256);
            }
            chess::Piece::WhiteBishop => {
                return pst::BISHOP_POSITIONAL_SCORE[square]
                    + (((365 * (256 - phase as i32)) + (297 * phase as i32)) / 256);
            }
            chess::Piece::WhiteRook => {
                return pst::ROOK_POSITIONAL_SCORE[square]
                    + (((477 * (256 - phase as i32)) + (512 * phase as i32)) / 256);
            }
            chess::Piece::WhiteQueen => {
                return pst::ROOK_POSITIONAL_SCORE[square]
                    + pst::BISHOP_POSITIONAL_SCORE[square]
                    + (((1025 * (256 - phase as i32)) + (936 * phase as i32)) / 256);
            }
            chess::Piece::WhiteKing => {
                return pst::KING_POSITIONAL_SCORE[square] + 12000;
            }
            _ => return 0,
        };
    } else {
        match piece {
            chess::Piece::BlackPawn => {
                return pst::PAWN_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize]
                    + (((82 * (256 - phase as i32)) + (94 * phase as i32)) / 256);
            }
            chess::Piece::BlackKnight => {
                return pst::KNIGHT_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize]
                    + (((337 * (256 - phase as i32)) + (281 * phase as i32)) / 256);
            }
            chess::Piece::BlackBishop => {
                return pst::BISHOP_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize]
                    + (((365 * (256 - phase as i32)) + (297 * phase as i32)) / 256);
            }
            chess::Piece::BlackRook => {
                return pst::ROOK_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize]
                    + (((477 * (256 - phase as i32)) + (512 * phase as i32)) / 256);
            }
            chess::Piece::BlackQueen => {
                return (pst::ROOK_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize]
                    + pst::BISHOP_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize])
                    + (((1025 * (256 - phase as i32)) + (936 * phase as i32)) / 256);
            }
            chess::Piece::BlackKing => {
                return pst::KING_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize] + 12000;
            }
            _ => return 0,
        }
    }
}
