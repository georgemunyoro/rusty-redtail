use crate::chess;

/*
   Represents a move in the game of chess. Data is stored in a single u32

    from      to        piece   capture  castle   promotion   en_passant
    00000000  00000000  0000    0000     0        0000        0

*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitPackedMove {
    pub move_bits: u32,
}

impl BitPackedMove {
    // This is how much we need to shift each value by to store
    // it and retrieve from the u32
    const FROM_SHIFT: u8 = 22;
    const TO_SHIFT: u8 = 14;
    const PIECE_SHIFT: u8 = 10;
    const CAPTURE_SHIFT: u8 = 6;
    const CASTLE_SHIFT: u8 = 5;
    const PROMOTION_SHIFT: u8 = 1;
    const ENPAS_SHIFT: u8 = 0;

    pub fn new(from: chess::Square, to: chess::Square, piece: chess::Piece) -> BitPackedMove {
        let mut bit_packed_move = BitPackedMove { move_bits: 0 };

        bit_packed_move.set_from(from);
        bit_packed_move.set_to(to);
        bit_packed_move.set_piece(piece);
        bit_packed_move.set_promotion(chess::Piece::Empty);
        bit_packed_move.set_capture(chess::Piece::Empty);

        return bit_packed_move;
    }

    pub fn default() -> BitPackedMove {
        let mut bit_packed_move = BitPackedMove { move_bits: 0 };
        bit_packed_move.set_piece(chess::Piece::Empty);
        bit_packed_move.set_promotion(chess::Piece::Empty);
        bit_packed_move.set_capture(chess::Piece::Empty);
        return bit_packed_move;
    }

    /*
       GETTERS
    */

    pub fn get_piece(&self) -> chess::Piece {
        return chess::Piece::from(((self.move_bits >> BitPackedMove::PIECE_SHIFT) as usize) & 0xF);
    }

    pub fn get_from(&self) -> chess::Square {
        return chess::Square::from((self.move_bits >> BitPackedMove::FROM_SHIFT) as usize);
    }

    pub fn get_to(&self) -> chess::Square {
        return chess::Square::from((self.move_bits >> BitPackedMove::TO_SHIFT) as usize & 0xFF);
    }

    pub fn get_promotion(&self) -> chess::Piece {
        return chess::Piece::from(
            (self.move_bits >> BitPackedMove::PROMOTION_SHIFT & 0xF) as usize,
        );
    }

    pub fn get_capture(&self) -> chess::Piece {
        return chess::Piece::from((self.move_bits >> BitPackedMove::CAPTURE_SHIFT & 0xF) as usize);
    }

    /*
       SETTERS
    */

    pub fn set_promotion(&mut self, promotion_piece: chess::Piece) {
        self.move_bits = (self.move_bits & !(0xF << BitPackedMove::PROMOTION_SHIFT))
            | ((promotion_piece as u32) << BitPackedMove::PROMOTION_SHIFT);
    }

    pub fn set_capture(&mut self, captured_piece: chess::Piece) {
        self.move_bits = (self.move_bits & !(0xF << BitPackedMove::CAPTURE_SHIFT))
            | (captured_piece as u32) << BitPackedMove::CAPTURE_SHIFT;
    }

    /// Sets the move castle to true
    pub fn set_castle(&mut self) {
        self.move_bits |= (1 as u32) << BitPackedMove::CASTLE_SHIFT;
    }

    /// Sets the move enpassant to true
    pub fn set_enpassant(&mut self) {
        self.move_bits |= (1 as u32) << BitPackedMove::ENPAS_SHIFT;
    }

    pub fn set_from(&mut self, from: chess::Square) {
        self.move_bits = (self.move_bits & !(0xFF << BitPackedMove::FROM_SHIFT))
            | (from as u32) << BitPackedMove::FROM_SHIFT;
    }

    pub fn set_to(&mut self, to: chess::Square) {
        self.move_bits = (self.move_bits & !(0xFF << BitPackedMove::TO_SHIFT))
            | (to as u32) << BitPackedMove::TO_SHIFT;
    }

    pub fn set_piece(&mut self, piece: chess::Piece) {
        self.move_bits = (self.move_bits & !(0xF << BitPackedMove::PIECE_SHIFT))
            | (piece as u32) << BitPackedMove::PIECE_SHIFT;
    }

    /*
       MISC
    */

    pub fn is_capture(&self) -> bool {
        return ((self.move_bits >> BitPackedMove::CAPTURE_SHIFT) & 0xF)
            != chess::Piece::Empty as u32;
    }

    pub fn is_promotion(&self) -> bool {
        return ((self.move_bits >> BitPackedMove::PROMOTION_SHIFT) & 0xF)
            != chess::Piece::Empty as u32;
    }

    pub fn is_castle(&self) -> bool {
        return ((self.move_bits >> BitPackedMove::CASTLE_SHIFT) & 0b1) == 1;
    }

    pub fn is_enpassant(&self) -> bool {
        return ((self.move_bits >> BitPackedMove::ENPAS_SHIFT) & 0b1) == 1;
    }
}

/*
   Moves are printed in uci format, e.g:
    - e2e4
    - a7a8q
    - h1g1
*/
impl std::fmt::Display for BitPackedMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let promotion = self.get_promotion();

        return write!(
            f,
            "{}{}{}",
            self.get_from().to_string().to_lowercase(),
            self.get_to().to_string().to_lowercase(),
            if self.is_promotion() {
                promotion.to_string().to_lowercase()
            } else {
                String::from("")
            }
        );
    }
}
