use std::sync::{Arc, Mutex};

use crate::{
    board::{Board, Position},
    chess,
    movegen::MoveGenerator,
    tt::{self, TranspositionTable},
    utils,
};

pub const MAX_PLY: usize = 64;

static ROOK_POSITIONAL_SCORE: [i32; 64] = [
    50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 0, 0, 10, 20, 20, 10, 0, 0, 0,
    0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20,
    20, 10, 0, 0, 0, 0, 0, 20, 20, 0, 0, 0,
];

static KNIGHT_POSITIONAL_SCORE: [i32; 64] = [
    -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 10, 10, 0, 0, -5, -5, 5, 20, 20, 20, 20, 5, -5, -5, 10, 20,
    30, 30, 20, 10, -5, -5, 10, 20, 30, 30, 20, 10, -5, -5, 5, 20, 10, 10, 20, 5, -5, -5, 0, 0, 0,
    0, 0, 0, -5, -5, -10, 0, 0, 0, 0, -10, -5,
];

static BISHOP_POSITIONAL_SCORE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 0, 0, 0, 0, 0, 10, 20, 20, 10,
    0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 10, 0, 0, 0, 0, 10, 0, 0, 30, 0, 0, 0, 0, 30, 0, 0, 0,
    -10, 0, 0, -10, 0, 0,
];

static PAWN_POSITIONAL_SCORE: [i32; 64] = [
    90, 90, 90, 90, 90, 90, 90, 90, 30, 30, 30, 40, 40, 30, 30, 30, 20, 20, 20, 30, 30, 30, 20, 20,
    10, 10, 10, 20, 20, 10, 10, 10, 5, 5, 10, 20, 20, 5, 5, 5, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0,
    -10, -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

static KING_POSITIONAL_SCORE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 0, 0, 0, 5, 5, 10, 10, 5, 5, 0, 0, 5, 10, 20, 20, 10,
    5, 0, 0, 5, 10, 20, 20, 10, 5, 0, 0, 0, 5, 10, 10, 5, 0, 0, 0, 5, 5, -5, -5, 0, 5, 0, 0, 0, 5,
    0, -15, 0, 10, 0,
];

static MVV_LVA: [[u32; 12]; 12] = [
    [105, 205, 305, 405, 505, 605, 105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604, 104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603, 103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602, 102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601, 101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600, 100, 200, 300, 400, 500, 600],
    [105, 205, 305, 405, 505, 605, 105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604, 104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603, 103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602, 102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601, 101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600, 100, 200, 300, 400, 500, 600],
];

static MIRROR_SCORE: [chess::Square; 64] = [
    chess::Square::A1,
    chess::Square::B1,
    chess::Square::C1,
    chess::Square::D1,
    chess::Square::E1,
    chess::Square::F1,
    chess::Square::G1,
    chess::Square::H1,
    chess::Square::A2,
    chess::Square::B2,
    chess::Square::C2,
    chess::Square::D2,
    chess::Square::E2,
    chess::Square::F2,
    chess::Square::G2,
    chess::Square::H2,
    chess::Square::A3,
    chess::Square::B3,
    chess::Square::C3,
    chess::Square::D3,
    chess::Square::E3,
    chess::Square::F3,
    chess::Square::G3,
    chess::Square::H3,
    chess::Square::A4,
    chess::Square::B4,
    chess::Square::C4,
    chess::Square::D4,
    chess::Square::E4,
    chess::Square::F4,
    chess::Square::G4,
    chess::Square::H4,
    chess::Square::A5,
    chess::Square::B5,
    chess::Square::C5,
    chess::Square::D5,
    chess::Square::E5,
    chess::Square::F5,
    chess::Square::G5,
    chess::Square::H5,
    chess::Square::A6,
    chess::Square::B6,
    chess::Square::C6,
    chess::Square::D6,
    chess::Square::E6,
    chess::Square::F6,
    chess::Square::G6,
    chess::Square::H6,
    chess::Square::A7,
    chess::Square::B7,
    chess::Square::C7,
    chess::Square::D7,
    chess::Square::E7,
    chess::Square::F7,
    chess::Square::G7,
    chess::Square::H7,
    chess::Square::A8,
    chess::Square::B8,
    chess::Square::C8,
    chess::Square::D8,
    chess::Square::E8,
    chess::Square::F8,
    chess::Square::G8,
    chess::Square::H8,
];

const REDUCTION_LIMIT: u8 = 3;
const FULL_DEPTH_MOVES: u8 = 4;

#[derive(Debug)]
pub struct PositionEvaluation {
    pub score: i32,
    pub best_move: Option<chess::Move>,
    pub depth: u8,
    pub ply: u32,
    pub nodes: i32,
}

#[derive(Debug)]
pub struct SearchOptions {
    pub depth: Option<u8>,
    pub movetime: Option<u32>,
    pub infinite: bool,
    pub wtime: Option<u32>,
    pub btime: Option<u32>,
    pub winc: Option<u32>,
    pub binc: Option<u32>,
    pub movestogo: Option<u32>,
}

impl SearchOptions {
    pub fn new() -> SearchOptions {
        return SearchOptions {
            depth: None,
            movetime: None,
            infinite: false,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
        };
    }
}

pub struct Evaluator {
    pub running: Arc<Mutex<bool>>,
    pub result: PositionEvaluation,
    pub killer_moves: [[chess::Move; MAX_PLY]; 2],
    pub history_moves: [[u32; MAX_PLY]; 12],
    pub pv_table: [[chess::Move; MAX_PLY]; MAX_PLY],
    pub pv_length: [u32; MAX_PLY],
    pub started_at: u128,
    pub options: SearchOptions,
    pub tt: TranspositionTable,
}

impl Evaluator {
    fn is_running(&mut self) -> bool {
        let r = self.running.lock().unwrap();
        return *r;
    }

    fn set_running(&mut self, b: bool) {
        let mut r = self.running.lock().unwrap();
        *r = b;
    }

    pub fn new() -> Evaluator {
        return Evaluator {
            running: Arc::new(Mutex::new(false)),
            result: PositionEvaluation {
                score: 0,
                best_move: None,
                depth: 0,
                ply: 0,
                nodes: 0,
            },
            killer_moves: [[chess::NULL_MOVE; MAX_PLY]; 2],
            history_moves: [[0; MAX_PLY]; 12],
            pv_length: [0; MAX_PLY],
            pv_table: [[chess::NULL_MOVE; MAX_PLY]; MAX_PLY],
            started_at: 0,
            options: SearchOptions {
                depth: None,
                movetime: None,
                infinite: false,
                binc: None,
                winc: None,
                btime: None,
                wtime: None,
                movestogo: None,
            },
            tt: TranspositionTable::new(),
        };
    }

    pub fn get_best_move(
        &mut self,
        position: &mut Position,
        options: SearchOptions,
        running: Arc<Mutex<bool>>,
    ) -> Option<chess::Move> {
        self.result = PositionEvaluation {
            score: -50000,
            best_move: None,
            depth: 0,
            ply: 0,
            nodes: 0,
        };
        self.killer_moves = [[chess::NULL_MOVE; MAX_PLY]; 2];
        self.history_moves = [[0; MAX_PLY]; 12];
        self.pv_length = [0; MAX_PLY];
        self.pv_table = [[chess::NULL_MOVE; MAX_PLY]; MAX_PLY];
        self.tt.clear();

        let depth = match options.depth {
            Some(depth) => depth as u8,
            None => MAX_PLY as u8,
        };

        let mut alpha = -50000;
        let mut beta = 50000;

        let mut current_depth = 1;

        self.options = options;
        self.started_at = self.get_time_ms();
        self.running = running;

        loop {
            if current_depth > depth {
                break;
            }

            self.result.ply = 0;
            let start_time = self.get_time_ms();

            let score = self.negamax(position, alpha, beta, current_depth);

            if (score <= alpha) || (score >= beta) {
                alpha = -50000;
                beta = 50000;
                continue;
            }

            alpha = score - 50;
            beta = score + 50;

            if !self.is_running() {
                break;
            }

            let stop_time = self.get_time_ms();
            let nps =
                (self.result.nodes as f64 / ((stop_time - start_time) as f64 / 1000.0)) as i32;

            match self.result.best_move {
                None => {}
                _ => {
                    let is_mate = self.result.score > 48000;
                    let mut mate_in: i32 = 0;

                    if is_mate {
                        let x = -(self.result.score - 49000);
                        if x % 2 == 0 {
                            mate_in = x as i32 / 2;
                        } else {
                            mate_in = (x as i32 + 1) / 2;
                        }
                    }

                    print!(
                        "info score {} {} depth {} nodes {} nps {}",
                        if is_mate { "mate" } else { "cp" },
                        if is_mate { mate_in } else { self.result.score },
                        self.result.depth,
                        self.result.nodes,
                        nps
                    );

                    let mut pv = String::new();
                    for i in 0..self.pv_length[0] {
                        let pv_node = self.pv_table[0][i as usize];
                        if pv_node == chess::NULL_MOVE {
                            break;
                        }
                        pv.push_str(pv_node.to_string().as_str());
                        pv.push_str(" ");
                    }
                    println!(" info pv {}", pv);
                }
            }

            current_depth += 1;
        }

        match self.result.best_move {
            Some(m) => {
                println!("bestmove {}", m);
            }
            None => {
                println!("bestmove {}", chess::NULL_MOVE)
            }
        }

        return self.result.best_move;
    }

    fn check_time(&self) -> bool {
        if self.options.infinite {
            return true;
        }

        let elapsed = self.get_time_ms() - self.started_at;

        match self.options.movetime {
            Some(movetime) => {
                if elapsed >= movetime as u128 {
                    return false;
                }
            }
            None => {}
        }

        match self.options.wtime {
            Some(wtime) => {
                if elapsed >= wtime as u128 {
                    return false;
                }
            }
            None => {}
        }

        match self.options.btime {
            Some(btime) => {
                if elapsed >= btime as u128 {
                    return false;
                }
            }
            None => {}
        }

        return true;
    }

    fn has_non_pawn_material(&self, position: &mut Position) -> bool {
        if position.turn == chess::Color::White {
            (position.bitboards[chess::Piece::WhiteBishop as usize]
                + position.bitboards[chess::Piece::WhiteKnight as usize]
                + position.bitboards[chess::Piece::WhiteRook as usize]
                + position.bitboards[chess::Piece::WhiteQueen as usize])
                != 0
        } else {
            (position.bitboards[chess::Piece::BlackBishop as usize]
                + position.bitboards[chess::Piece::BlackKnight as usize]
                + position.bitboards[chess::Piece::BlackRook as usize]
                + position.bitboards[chess::Piece::BlackQueen as usize])
                != 0
        }
    }

    fn get_time_ms(&self) -> u128 {
        let now = std::time::SystemTime::now();
        let since_the_epoch = now
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        return since_the_epoch.as_millis();
    }

    fn negamax(&mut self, position: &mut Position, alpha: i32, beta: i32, depth: u8) -> i32 {
        if self.result.nodes & 2047 == 0 {
            self.set_running(self.check_time());
        }

        if let Some(stored_value) = self.tt.probe(position.hash, depth, alpha, beta) {
            return stored_value;
        }

        let mut transposition_hash_flag = tt::TranspositionTableEntryFlag::ALPHA;

        self.pv_length[self.result.ply as usize] = self.result.ply;

        if depth == 0 {
            let eval = self.quiescence(position, alpha, beta);

            self.tt.save(
                position.hash,
                depth,
                tt::TranspositionTableEntryFlag::EXACT,
                eval,
            );
            return eval;
        }

        if depth >= 3
            && !position.is_in_check()
            && self.result.ply > 0
            && self.has_non_pawn_material(position)
        {
            position.make_null_move();
            let null_move_value = -self.negamax(position, -beta, -beta + 1, depth - 1 - 2);
            position.unmake_move();

            if null_move_value >= beta {
                return beta;
            }
        }

        if self.result.ply > (MAX_PLY as u32) {
            return self.evaluate(position);
        }

        self.result.nodes += 1;

        let mut alpha_mut = alpha.clone();
        let mut moves = position.generate_moves();

        let is_following_pv_line = moves
            .iter()
            .any(|&m| m == self.pv_table[0][self.result.ply as usize]);
        self.order_moves(&mut moves, is_following_pv_line);

        let mut legal_move_count = 0;
        let mut found_pv = false;
        let mut moves_searched = 0;

        for m in moves {
            let is_valid = position.make_move(m, false);
            if !is_valid {
                continue;
            }

            legal_move_count += 1;
            self.result.ply += 1;

            let score: i32 = if found_pv {
                let val = -self.negamax(position, -alpha_mut - 1, -alpha_mut, depth);
                if val > alpha_mut && val < beta {
                    -self.negamax(position, -beta, -alpha_mut, depth - 1)
                } else {
                    val
                }
            } else {
                if moves_searched == 0 {
                    -self.negamax(position, -beta, -alpha_mut, depth - 1)
                } else {
                    // LMR
                    let mut lmr_score = if moves_searched > FULL_DEPTH_MOVES
                        && depth >= REDUCTION_LIMIT
                        && !position.is_in_check()
                        && m.capture == None
                        && m.promotion == None
                    {
                        -self.negamax(position, -alpha_mut - 1, -alpha_mut, depth - 2)
                    } else {
                        alpha_mut + 1
                    };

                    if lmr_score > alpha_mut {
                        lmr_score = -self.negamax(position, -alpha_mut - 1, -alpha_mut, depth - 1);

                        if lmr_score > alpha_mut && lmr_score < beta {
                            lmr_score = -self.negamax(position, -beta, -alpha_mut, depth - 1);
                        }
                    }

                    lmr_score
                }
            };

            self.result.ply -= 1;

            position.unmake_move();

            moves_searched += 1;

            if !self.is_running() {
                return 0;
            }

            if score >= beta {
                match m.capture {
                    None => {
                        self.killer_moves[1][self.result.ply as usize] =
                            self.killer_moves[0][self.result.ply as usize];
                        self.killer_moves[0][self.result.ply as usize] = m;
                    }
                    _ => {}
                }

                self.tt.save(
                    position.hash,
                    depth,
                    tt::TranspositionTableEntryFlag::BETA,
                    beta,
                );

                return beta;
            }

            if score > alpha_mut {
                found_pv = true;

                transposition_hash_flag = tt::TranspositionTableEntryFlag::EXACT;

                self.history_moves[m.piece as usize][m.to as usize] += depth as u32;

                self.pv_table[self.result.ply as usize][self.result.ply as usize] = m;
                for i in (self.result.ply + 1)..(self.pv_length[self.result.ply as usize + 1]) {
                    self.pv_table[self.result.ply as usize][i as usize] =
                        self.pv_table[self.result.ply as usize + 1][i as usize];
                }

                self.pv_length[self.result.ply as usize] =
                    self.pv_length[self.result.ply as usize + 1];

                alpha_mut = score;
                if self.result.ply == 0 {
                    self.result.best_move = Some(m);
                    self.result.score = score;
                    self.result.depth = depth;
                }
            }
        }

        if legal_move_count == 0 {
            if position.is_in_check() {
                return -49000 + (self.result.ply as i32);
            } else {
                return 0;
            }
        }

        self.tt
            .save(position.hash, depth, transposition_hash_flag, alpha_mut);

        return alpha_mut;
    }

    fn quiescence(&mut self, position: &mut Position, alpha: i32, beta: i32) -> i32 {
        if self.result.nodes & 2047 == 0 {
            self.set_running(self.check_time());
        }

        self.result.nodes += 1;

        let mut mut_alpha = alpha.clone();

        let evaluation = self.evaluate(position);

        if evaluation >= beta {
            return beta;
        }

        if evaluation > mut_alpha {
            mut_alpha = evaluation;
        }

        let mut moves = position.generate_moves();
        self.order_moves(&mut moves, false);

        for m in moves {
            let is_valid = position.make_move(m, true);
            if !is_valid {
                continue;
            }

            self.result.ply += 1;

            let score = -self.quiescence(position, -beta, -mut_alpha);

            self.result.ply -= 1;

            position.unmake_move();

            if !self.is_running() {
                return 0;
            }

            if score >= beta {
                return beta;
            }

            if score > mut_alpha {
                mut_alpha = score;
            }
        }

        return mut_alpha;
    }

    /// Returns a score for a move based on the Most Valuable Victim - Least Valuable Attacker heuristic.
    fn get_move_mvv_lva(&mut self, m: chess::Move, is_following_pv_line: bool) -> u32 {
        if is_following_pv_line {
            if m == self.pv_table[0][self.result.ply as usize] {
                return 20000;
            }
        }

        let value = match m.capture {
            Some(c) => MVV_LVA[m.piece as usize][c as usize],
            None => {
                if self.killer_moves[0][self.result.ply as usize] == m {
                    return 9000;
                }

                if self.killer_moves[1][self.result.ply as usize] == m {
                    return 9000;
                }

                return self.history_moves[m.piece as usize][m.to as usize];
            }
        };

        return value;
    }

    fn order_moves(&mut self, moves: &mut Vec<chess::Move>, is_following_pv_line: bool) {
        moves.sort_by(|a, b| {
            let a_value = self.get_move_mvv_lva(*a, is_following_pv_line);
            let b_value = self.get_move_mvv_lva(*b, is_following_pv_line);
            return b_value.cmp(&a_value);
        });
    }

    fn get_piece_value(&mut self, piece: chess::Piece, square: usize) -> i32 {
        match piece {
            chess::Piece::WhitePawn => {
                let mut score = 100;
                score += PAWN_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteKnight => {
                let mut score = 300;
                score += KNIGHT_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteBishop => {
                let mut score = 350;
                score += BISHOP_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteRook => {
                let mut score = 500;
                score += ROOK_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteQueen => {
                let mut score = 1000;
                score += ROOK_POSITIONAL_SCORE[square] + BISHOP_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteKing => {
                let mut score = 10000;
                score += KING_POSITIONAL_SCORE[square];
                return score;
            }

            chess::Piece::BlackPawn => {
                let mut score = -100;
                score -= PAWN_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackKnight => {
                let mut score = -300;
                score -= KNIGHT_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackBishop => {
                let mut score = -350;
                score -= BISHOP_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackRook => {
                let mut score = -500;
                score -= ROOK_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackQueen => {
                let mut score = -1000;
                score -= ROOK_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize]
                    + BISHOP_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackKing => {
                let mut score = -10000;
                score -= KING_POSITIONAL_SCORE[MIRROR_SCORE[square] as usize];
                return score;
            }

            chess::Piece::Empty => 0,
        }
    }

    fn evaluate(&mut self, position: &mut Position) -> i32 {
        let mut score = 0;

        for piece in (chess::Piece::WhitePawn as usize)..=(chess::Piece::BlackKing as usize) {
            let piece = chess::Piece::from(piece);
            let mut bitboard = position.bitboards[piece as usize];

            while bitboard != 0 {
                let square = utils::pop_lsb(&mut bitboard);
                score += self.get_piece_value(piece, square as usize);
            }
        }

        return if position.turn == chess::Color::White {
            score
        } else {
            -score
        };
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         board::{Board, Position},
//         chess,
//         evaluation::Evaluator,
//     };

//     #[test]
//     fn evaluates_correctly() {
//         let mut evaluator = Evaluator {
//             running: false,
//             transposition_table: std::collections::HashMap::new(),
//             result: crate::evaluation::PositionEvaluation {
//                 score: 0,
//                 best_move: None,
//                 depth: 0,
//                 ply: 0,
//                 nodes: 0,
//             },
//             killer_moves: [[chess::NULL_MOVE; 64]; 2],
//             history_moves: [[0; 64]; 12],
//             pv_length: [0; 64],
//             pv_table: [[chess::NULL_MOVE; 64]; 64],
//             started_at: 0,
//             options: crate::evaluation::SearchOptions {
//                 depth: None,
//                 movetime: None,
//                 infinite: false,
//                 wtime: None,
//                 btime: None,
//                 winc: None,
//                 binc: None,
//                 movestogo: None,
//             },
//             tt: std::collections::HashMap::new(),
//         };
//         let mut board = Position::new(None);

//         board.set_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == 0);

//         board.set_fen("rnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNBQKBNR w KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == -100);

//         board.set_fen("rnbqkbnr/pppppppp/8/8/8/8/3PPPPP/RNBQKBNR w KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == -300);

//         board.set_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/1NBQKBNR w KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == -500);

//         board.set_fen("rnbqkbn1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == 500);

//         board.set_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RN1QKBNR w KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == -340);

//         board.set_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == 30);

//         board.set_fen("rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == 30);

//         board.set_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == 0);

//         board.set_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == 30);

//         board.set_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 0 1");
//         assert!(evaluator.evaluate(&mut board) == -30);
//     }
// }
