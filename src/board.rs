use crate::{chess, utils};

/// A chess position
pub struct Position {
    bitboards: [u64; 12],
    turn: chess::Color,

    pawn_attacks: [[u64; 64]; 2],
    knight_attacks: [u64; 64],
    king_attacks: [u64; 64],

    bishop_attacks: [u64; 64],
    rook_attacks: [u64; 64],
    queen_attacks: [u64; 64],
}

pub trait Board {
    fn new(fen: Option<&str>) -> Position;
    fn draw(&mut self);
    fn set_fen(&mut self, fen: &str);

    fn debug(&mut self);
}

impl Board for Position {
    fn new(fen: Option<&str>) -> Position {
        let mut pos = Position {
            bitboards: [0; 12],
            turn: chess::Color::Both,

            pawn_attacks: [[0; 64]; 2],
            knight_attacks: [0; 64],
            king_attacks: [0; 64],
            bishop_attacks: [0; 64],
            rook_attacks: [0; 64],
            queen_attacks: [0; 64],
        };

        pos.initialize_leaper_piece_attacks();
        pos.initialize_slider_piece_attacks();

        match fen {
            Some(p) => Position::set_fen(&mut pos, p),
            None => {}
        }

        return pos;
    }

    /// Draws the board to the console
    fn draw(&mut self) {
        for i in 0..64 {
            if i % 8 == 0 {
                println!();
            }
            print!("{} ", self.get_piece_at_square(i));
        }
        println!("\n\n{} to move\n", self.turn);
    }

    /// Sets the board to the given FEN string
    fn set_fen(&mut self, fen: &str) {
        // Split the fen
        let sections = fen.split(' ').collect::<Vec<&str>>();

        // Clear the bitboards
        for i in 1..12 {
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

            pos += 1;
        }

        // Set the turn
        self.turn = chess::Color::from(sections[1].chars().next().unwrap());
    }

    fn debug(&mut self) {
        let mut blockers = 0u64;

        // utils::set_bit(&mut blockers, chess::Square::D7 as u8);
        // utils::set_bit(&mut blockers, chess::Square::D2 as u8);
        // utils::set_bit(&mut blockers, chess::Square::D1 as u8);
        // utils::set_bit(&mut blockers, chess::Square::B4 as u8);
        // utils::set_bit(&mut blockers, chess::Square::G4 as u8);

        // utils::print_bitboard(self.generate_rook_attacks_on_the_fly(chess::Square::D4, blockers));

        for i in 0..4096 {
            let attack_mask = self.mask_rook_attacks(chess::Square::A1);
            let occupancy = self.set_occupancy(i, utils::count_bits(attack_mask), attack_mask);
            utils::print_bitboard(occupancy)
        }
    }
}

impl Position {
    fn get_piece_at_square(&self, square: u8) -> chess::Piece {
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

    fn set_occupancy(&self, index: i32, bits_in_mask: u64, attack_mask: u64) -> u64 {
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
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Position},
        chess::Piece,
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
}
