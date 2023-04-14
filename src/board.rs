pub mod board {
    pub struct Position {
        bitboards: [u64; 12],
        en_passant: u64,
        castling: u8,
        turn: u8,
        halfmove_clock: u8,
        fullmove_number: u16,
    }

    pub trait Board {
        fn new() -> Position {
            Position {
                bitboards: [0; 12],
                en_passant: 0,
                castling: 0,
                turn: 0,
                halfmove_clock: 0,
                fullmove_number: 0,
            }
        }
    }
}
