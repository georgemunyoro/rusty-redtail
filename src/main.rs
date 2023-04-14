use std::fmt::Display;

/// Represents a chess piece
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Piece {
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,

    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,

    Empty,
}

static PIECE_ITER: [Piece; 12] = [
    Piece::BlackPawn,
    Piece::BlackKnight,
    Piece::BlackBishop,
    Piece::BlackRook,
    Piece::BlackQueen,
    Piece::BlackKing,
    Piece::WhitePawn,
    Piece::WhiteKnight,
    Piece::WhiteBishop,
    Piece::WhiteRook,
    Piece::WhiteQueen,
    Piece::WhiteKing,
];

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_piece_char(*self))
    }
}

impl Into<char> for Piece {
    fn into(self) -> char {
        get_piece_char(self)
    }
}

impl From<char> for Piece {
    fn from(v: char) -> Self {
        match v {
            'p' => Piece::BlackPawn,
            'n' => Piece::BlackKnight,
            'b' => Piece::BlackBishop,
            'r' => Piece::BlackRook,
            'q' => Piece::BlackQueen,
            'k' => Piece::BlackKing,
            'P' => Piece::WhitePawn,
            'N' => Piece::WhiteKnight,
            'B' => Piece::WhiteBishop,
            'R' => Piece::WhiteRook,
            'Q' => Piece::WhiteQueen,
            'K' => Piece::WhiteKing,
            '.' => Piece::Empty,
            _ => panic!("Invalid piece"),
        }
    }
}

impl From<Piece> for usize {
    fn from(v: Piece) -> Self {
        match v {
            Piece::BlackPawn => 0,
            Piece::BlackKnight => 1,
            Piece::BlackBishop => 2,
            Piece::BlackRook => 3,
            Piece::BlackQueen => 4,
            Piece::BlackKing => 5,
            Piece::WhitePawn => 6,
            Piece::WhiteKnight => 7,
            Piece::WhiteBishop => 8,
            Piece::WhiteRook => 9,
            Piece::WhiteQueen => 10,
            Piece::WhiteKing => 11,
            Piece::Empty => 12,
        }
    }
}

impl From<usize> for Piece {
    fn from(v: usize) -> Self {
        match v {
            0 => Piece::BlackPawn,
            1 => Piece::BlackKnight,
            2 => Piece::BlackBishop,
            3 => Piece::BlackRook,
            4 => Piece::BlackQueen,
            5 => Piece::BlackKing,
            6 => Piece::WhitePawn,
            7 => Piece::WhiteKnight,
            8 => Piece::WhiteBishop,
            9 => Piece::WhiteRook,
            10 => Piece::WhiteQueen,
            11 => Piece::WhiteKing,
            12 => Piece::Empty,
            _ => panic!("Invalid piece"),
        }
    }
}

/// Represents the color of a piece
enum Color {
    White,
    Black,
    Both,
}

/// Returns the character representation of a piece
fn get_piece_char(piece: Piece) -> char {
    match piece {
        Piece::BlackPawn => 'p',
        Piece::BlackKnight => 'n',
        Piece::BlackBishop => 'b',
        Piece::BlackRook => 'r',
        Piece::BlackQueen => 'q',
        Piece::BlackKing => 'k',
        Piece::WhitePawn => 'P',
        Piece::WhiteKnight => 'N',
        Piece::WhiteBishop => 'B',
        Piece::WhiteRook => 'R',
        Piece::WhiteQueen => 'Q',
        Piece::WhiteKing => 'K',
        Piece::Empty => '.',
    }
}

/// A chess position
pub struct Position {
    bitboards: [u64; 12],
    turn: Color,
}

mod utils {
    pub fn get_bit(bitboard: u64, bit_index: u8) -> u64 {
        return (bitboard) & (1 << bit_index);
    }

    pub fn set_bit(bitboard: u64, bit_index: u8) -> u64 {
        return bitboard | (1 << bit_index);
    }

    pub fn _print_bitboard(bitboard: u64) {
        for i in 0..64 {
            if i % 8 == 0 {
                println!();
            }
            print!("{} ", get_bit(bitboard, i));
        }
        println!()
    }
}

impl Position {
    fn get_piece_at_square(&self, square: u8) -> Piece {
        for piece in PIECE_ITER {
            if utils::get_bit(self.bitboards[piece as usize], square) != 0 {
                return piece;
            }
        }
        return Piece::Empty;
    }

    /// Draws the board to the console
    fn draw(&mut self) {
        for i in 0..64 {
            if i % 8 == 0 {
                println!();
            }

            print!("{} ", self.get_piece_at_square(i));
        }

        println!(
            "\n\n{} to move\n",
            match self.turn {
                Color::White => "White",
                Color::Black => "Black",
                Color::Both => "Both",
            }
        );
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

            self.bitboards[Piece::from(c) as usize] =
                utils::set_bit(self.bitboards[Piece::from(c) as usize], pos);

            pos += 1;
        }

        // Set the turn
        self.turn = match sections[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Invalid turn"),
        };
    }
}

fn main() {
    let mut main_board = Position {
        bitboards: [0; 12],
        turn: Color::Both,
    };
    main_board.set_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    main_board.draw();
}

// -------------------------------------------------
// --------------------- TESTS ---------------------
// -------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::{get_piece_char, Color, Piece, Position};

    #[test]
    fn get_piece_char_gets_the_correct_character() {
        assert_eq!(get_piece_char(Piece::BlackPawn), 'p');
        assert_eq!(get_piece_char(Piece::BlackKnight), 'n');
        assert_eq!(get_piece_char(Piece::BlackBishop), 'b');
        assert_eq!(get_piece_char(Piece::BlackRook), 'r');
        assert_eq!(get_piece_char(Piece::BlackQueen), 'q');
        assert_eq!(get_piece_char(Piece::BlackKing), 'k');
        assert_eq!(get_piece_char(Piece::WhitePawn), 'P');
        assert_eq!(get_piece_char(Piece::WhiteKnight), 'N');
        assert_eq!(get_piece_char(Piece::WhiteBishop), 'B');
        assert_eq!(get_piece_char(Piece::WhiteRook), 'R');
        assert_eq!(get_piece_char(Piece::WhiteQueen), 'Q');
        assert_eq!(get_piece_char(Piece::WhiteKing), 'K');
        assert_eq!(get_piece_char(Piece::Empty), '.');
    }

    #[test]
    fn get_piece_at_square_and_set_fen_works() {
        let mut position = Position {
            bitboards: [0; 12],
            turn: Color::Both,
        };
        position.set_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
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
