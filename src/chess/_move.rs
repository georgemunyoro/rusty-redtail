use std::cmp::Ordering;

use crate::chess::{piece::Piece, square::Square};

/*
   Represents a move in the game of chess. Data is stored in a single u32.

    from      to        piece   capture  castle   promotion   en_passant
    00000000  00000000  0000    0000     0        0000        0

*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitPackedMove {
    pub move_bits: u32,
}

impl BitPackedMove {
    // This is how much we need to shift each value
    // by to store it and retrieve from the u32

    const FROM_SHIFT: u8 = 22;
    const TO_SHIFT: u8 = 14;
    const PIECE_SHIFT: u8 = 10;
    const CAPTURE_SHIFT: u8 = 6;
    const CASTLE_SHIFT: u8 = 5;
    const PROMOTION_SHIFT: u8 = 1;
    const ENPAS_SHIFT: u8 = 0;

    /// Returns a move, given the current square of the moving piece, the square to which it will
    /// move to, and the actual piece itself. This constructs the most basic type of move.
    pub fn new(from: Square, to: Square, piece: Piece) -> BitPackedMove {
        let mut bit_packed_move = BitPackedMove { move_bits: 0 };
        bit_packed_move.set_from(from);
        bit_packed_move.set_to(to);
        bit_packed_move.set_piece(piece);
        bit_packed_move.set_promotion(Piece::Empty);
        bit_packed_move.set_capture(Piece::Empty);
        return bit_packed_move;
    }

    /// Returns an invalid nullmove, it is an empty piece moving to an empty square from an empty
    /// square. Use this instead of Option<>
    pub fn default() -> BitPackedMove {
        let mut bit_packed_move = BitPackedMove { move_bits: 0 };
        bit_packed_move.set_piece(Piece::Empty);
        bit_packed_move.set_promotion(Piece::Empty);
        bit_packed_move.set_capture(Piece::Empty);
        return bit_packed_move;
    }

    /*
       GETTERS
    */

    /// Returns the piece being moved
    pub fn get_piece(&self) -> Piece {
        return Piece::from(((self.move_bits >> BitPackedMove::PIECE_SHIFT) as usize) & 0xF);
    }

    /// Returns the initial square of the piece moving
    pub fn get_from(&self) -> Square {
        return Square::from((self.move_bits >> BitPackedMove::FROM_SHIFT) as usize);
    }

    /// Returns the square that the piece will move to
    pub fn get_to(&self) -> Square {
        return Square::from((self.move_bits >> BitPackedMove::TO_SHIFT) as usize & 0xFF);
    }

    /// If the move results in a promotion, returns the resulting piece being
    /// promoted to, otherwise returns an empty piece
    pub fn get_promotion(&self) -> Piece {
        return Piece::from((self.move_bits >> BitPackedMove::PROMOTION_SHIFT & 0xF) as usize);
    }

    /// Returns the piece being captured, if not a capturing move, returns an empty piece
    pub fn get_capture(&self) -> Piece {
        return Piece::from((self.move_bits >> BitPackedMove::CAPTURE_SHIFT & 0xF) as usize);
    }

    /*
       SETTERS
    */

    /// Sets the piece that will be promoted to
    pub fn set_promotion(&mut self, promotion_piece: Piece) {
        self.move_bits = (self.move_bits & !(0xF << BitPackedMove::PROMOTION_SHIFT))
            | ((promotion_piece as u32) << BitPackedMove::PROMOTION_SHIFT);
    }

    /// Sets the piece that is being captured
    pub fn set_capture(&mut self, captured_piece: Piece) {
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

    /// Sets the inital square of the moving piece
    pub fn set_from(&mut self, from: Square) {
        self.move_bits = (self.move_bits & !(0xFF << BitPackedMove::FROM_SHIFT))
            | (from as u32) << BitPackedMove::FROM_SHIFT;
    }

    /// Sets the square that the moving piece is moving to
    pub fn set_to(&mut self, to: Square) {
        self.move_bits = (self.move_bits & !(0xFF << BitPackedMove::TO_SHIFT))
            | (to as u32) << BitPackedMove::TO_SHIFT;
    }

    /// Sets the piece that will be moving
    pub fn set_piece(&mut self, piece: Piece) {
        self.move_bits = (self.move_bits & !(0xF << BitPackedMove::PIECE_SHIFT))
            | (piece as u32) << BitPackedMove::PIECE_SHIFT;
    }

    /*
       MISC
    */

    /// Returns true if the move is a capture, otherwise false
    pub fn is_capture(&self) -> bool {
        return ((self.move_bits >> BitPackedMove::CAPTURE_SHIFT) & 0xF) != Piece::Empty as u32;
    }

    /// Returns true if the move results in a promotion, otherwise false
    pub fn is_promotion(&self) -> bool {
        return ((self.move_bits >> BitPackedMove::PROMOTION_SHIFT) & 0xF) != Piece::Empty as u32;
    }

    /// Returns true if the move is a castling move, otherwise false
    pub fn is_castle(&self) -> bool {
        return ((self.move_bits >> BitPackedMove::CASTLE_SHIFT) & 0b1) == 1;
    }

    /// Returns true if the move is an enpassant move, otherwise false
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

#[derive(Clone, Copy, Debug)]
pub struct PrioritizedMove {
    pub priority: u32,
    pub m: BitPackedMove,
}

impl PrioritizedMove {
    pub fn new(m: BitPackedMove, priority: u32) -> Self {
        Self { priority, m }
    }

    pub fn default() -> Self {
        Self {
            priority: 0,
            m: BitPackedMove::default(),
        }
    }
}

impl PartialEq for PrioritizedMove {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedMove {}

impl Ord for PrioritizedMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for PrioritizedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
