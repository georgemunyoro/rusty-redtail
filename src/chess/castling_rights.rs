/// Represents the castling rights of a position
#[derive(Clone, Copy, Debug)]
pub struct CastlingRights {
    /*
       0b0001 => White can castle kingside
       0b0010 => White can castle queenside
       0b0100 => Black can castle kingside
       0b1000 => Black can castle queenside
    */
    rights: u8,
}

impl CastlingRights {
    // Constants for the castling rights:w
    pub const WHITE_KINGSIDE: u8 = 0b0001;
    pub const WHITE_QUEENSIDE: u8 = 0b0010;
    pub const BLACK_KINGSIDE: u8 = 0b0100;
    pub const BLACK_QUEENSIDE: u8 = 0b1000;

    /// Create a new castling rights with all rights
    pub fn new() -> Self {
        CastlingRights { rights: 0b1111 }
    }

    /// Create a new empty castling rights
    pub fn new_empty() -> Self {
        CastlingRights { rights: 0 }
    }

    /// Check if the current castling rights contain a right
    pub fn can_castle(&self, right: u8) -> bool {
        self.rights & right != 0
    }

    /// Remove a right from the current castling rights
    pub fn remove_right(&mut self, right: u8) {
        self.rights &= !right;
    }

    /// Add a right to the current castling rights
    pub fn add_right(&mut self, right: u8) {
        self.rights |= right;
    }

    /// Get the castling rights as a u8
    pub fn get_rights_u8(&self) -> u8 {
        self.rights
    }
}

impl From<&str> for CastlingRights {
    fn from(v: &str) -> Self {
        let mut castling_rights = CastlingRights::new_empty();
        for c in v.chars() {
            match c {
                'K' => castling_rights.add_right(CastlingRights::WHITE_KINGSIDE),
                'Q' => castling_rights.add_right(CastlingRights::WHITE_QUEENSIDE),
                'k' => castling_rights.add_right(CastlingRights::BLACK_KINGSIDE),
                'q' => castling_rights.add_right(CastlingRights::BLACK_QUEENSIDE),
                '-' => break,
                _ => panic!("Invalid castling rights"),
            }
        }
        return castling_rights;
    }
}
