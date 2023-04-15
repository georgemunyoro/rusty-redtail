use crate::{board::Position, chess, utils};

pub trait MoveGenerator {
    fn generate_moves(&self) -> Vec<chess::Move>;
    fn generate_pawn_moves(&self) -> Vec<chess::Move>;
}

impl MoveGenerator for Position {
    fn generate_pawn_moves(&self) -> Vec<chess::Move> {
        let mut moves = Vec::new();

        // white pawn moves
        if self.turn == chess::Color::White {
            let piece = chess::Piece::WhitePawn;
            let mut piece_bitboard = self.bitboards[piece as usize];

            while piece_bitboard != 0 {
                let source = chess::Square::from(utils::pop_lsb(&mut piece_bitboard));
                let target = chess::Square::from((source as u8) - 8);

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
                                let mut m =
                                    chess::Move::new(source, target, chess::Piece::WhitePawn);
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
                                moves.push(chess::Move::new(
                                    source,
                                    target,
                                    chess::Piece::WhitePawn,
                                ));
                            }
                        }
                    }
                }

                // pawn captures
                let mut attacks = self.pawn_attacks[chess::Color::White as usize][source as usize];
                while attacks != 0 {
                    let target = chess::Square::from(utils::get_lsb(attacks));

                    if source >= chess::Square::A7 && source <= chess::Square::H7 {
                    } else {
                        moves.push(chess::Move::new(source, target, chess::Piece::WhitePawn));
                    }

                    utils::clear_bit(&mut attacks, target as u8);
                }
            }
        }
        // black pawn moves
        else if self.turn == chess::Color::Black {
            let piece = chess::Piece::BlackPawn;
            let mut piece_bitboard = self.bitboards[piece as usize];

            while piece_bitboard != 0 {
                let source = chess::Square::from(utils::pop_lsb(&mut piece_bitboard));
                let target = chess::Square::from((source as u8) - 8);

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
                                let mut m =
                                    chess::Move::new(source, target, chess::Piece::BlackPawn);
                                m.promotion = Some(*promotion);
                                return m;
                            }),
                        );
                    } else {
                        // single pawn push
                        moves.push(chess::Move::new(source, target, chess::Piece::BlackPawn));

                        // double pawn push
                        if (source >= chess::Square::A2) && (source <= chess::Square::H2) {
                            let target = chess::Square::from((source as u8) - 16);
                            if self.get_piece_at_square(target as u8) == chess::Piece::Empty {
                                moves.push(chess::Move::new(
                                    source,
                                    target,
                                    chess::Piece::BlackPawn,
                                ));
                            }
                        }
                    }
                }

                // pawn captures
            }
        }

        return moves;
    }

    fn generate_moves(&self) -> Vec<chess::Move> {
        let mut moves = Vec::new();

        // pawn moves
        moves.extend(self.generate_pawn_moves());

        // knight moves

        // bishop moves

        // rook moves

        // queen moves

        // king moves

        return moves;
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::board::Board;

    #[derive(Serialize, Deserialize, Debug)]
    struct TestStart {
        fen: String,
        description: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestExpected {
        fen: String,
        r#move: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestCase {
        start: TestStart,
        expected: Vec<TestExpected>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestData {
        description: String,
        test_cases: Vec<TestCase>,
    }

    #[test]
    fn test_generate_moves() {
        let board = Position::new(Some(String::from(chess::constants::STARTING_FEN).as_str()));
        let moves = board.generate_moves();
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_pawn_move_generation() {
        let file = File::open("src/testdata/pawns.json").unwrap();
        let test_data: TestData = serde_json::from_reader(file).unwrap();
        println!("{:?}", test_data.description);

        for test_case in test_data.test_cases {
            let mut board = Position::new(Some(test_case.start.fen.as_str()));

            println!("--------------------------------------------------------------------");
            println!("{}", test_case.start.description);
            println!("{}", test_case.start.fen);
            println!("--------------------------------------------------------------------");

            board.draw();

            let moves = board.generate_moves();
            assert_eq!(moves.len(), test_case.expected.len());

            println!("--------------------------------------------------------------------");
        }
    }
}
