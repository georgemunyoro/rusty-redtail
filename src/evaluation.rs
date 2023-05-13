use std::sync::{Arc, Mutex};

use crate::{
    board::{Board, Position},
    chess,
    movegen::MoveGenerator,
    pst,
    pv::PrincipalVariationTable,
    tt::TranspositionTable,
    utils,
};

pub const MAX_PLY: usize = 64;

static _MVV_LVA: [[u32; 12]; 12] = [
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

const _REDUCTION_LIMIT: u8 = 3;
const _FULL_DEPTH_MOVES: u8 = 4;

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
    pub tt: Arc<Mutex<TranspositionTable>>,
    pub repetition_table: Vec<u64>,
    pub pv: PrincipalVariationTable,
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
            tt: Arc::new(Mutex::new(TranspositionTable::new())),
            repetition_table: Vec::with_capacity(150),
            pv: PrincipalVariationTable::new(),
        };
    }

    pub fn get_best_move(
        &mut self,
        position: &mut Position,
        options: SearchOptions,
        running: Arc<Mutex<bool>>,
        transposition_table: Arc<Mutex<TranspositionTable>>,
    ) -> Option<chess::Move> {
        self.result = PositionEvaluation {
            score: 0,
            best_move: None,
            depth: 0,
            ply: 0,
            nodes: 0,
        };

        let depth = match options.depth {
            Some(depth) => depth as u8,
            None => MAX_PLY as u8,
        };

        self.options = options;
        self.running = running;
        self.tt = transposition_table;
        self.started_at = self._get_time_ms();

        let alpha = -50000;
        let beta = 50000;
        let mut current_depth = 1;

        loop {
            if current_depth > depth {
                break;
            }

            self.result.nodes = 0;
            let start_time = self._get_time_ms();

            self.negamax(position, alpha, beta, current_depth);

            if !self.is_running() {
                break;
            }

            self.print_info(position, start_time);
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

    fn negamax(&mut self, position: &mut Position, _alpha: i32, beta: i32, depth: u8) -> i32 {
        let mut alpha = _alpha;

        if self.result.nodes & 2047 == 0 {
            self.set_running(self.check_time());
        }

        if depth == 0 {
            return self.evaluate(position);
        }

        let moves = position.generate_moves();
        for m in moves {
            let is_legal_move = position.make_move(m, false);
            if !is_legal_move {
                continue;
            }

            self.result.ply += 1;
            let score = -self.negamax(position, -beta, -alpha, depth - 1);
            self.result.ply -= 1;

            position.unmake_move();

            if !self.is_running() {
                return 0;
            }

            if score >= beta {
                return beta;
            }

            if score > alpha {
                self.pv.store(position.hash, m);
                if self.result.ply == 0 {
                    self.result.depth = depth;
                    self.result.score = score;
                    self.result.best_move = Some(m);
                }

                alpha = score;
            }
        }

        return alpha;
    }

    fn print_info(&self, position: &mut Position, start_time: u128) {
        let stop_time = self._get_time_ms();
        let nps = (self.result.nodes as f64 / ((stop_time - start_time) as f64 / 1000.0)) as i32;

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

                let mut pv_str = String::new();
                let pv_line_found = self.pv.get_pv_line(position);

                for i in pv_line_found {
                    pv_str.push_str(" ");
                    pv_str.push_str(i.to_string().as_str());
                }

                println!(" info pv{}", pv_str);
            }
        }
    }

    fn check_time(&self) -> bool {
        if self.options.infinite {
            return true;
        }

        let elapsed = self._get_time_ms() - self.started_at;

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

    fn _has_non_pawn_material(&self, position: &mut Position) -> bool {
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

    fn _get_time_ms(&self) -> u128 {
        let now = std::time::SystemTime::now();
        let since_the_epoch = now
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        return since_the_epoch.as_millis();
    }

    /// Returns a score for a move based on the Most Valuable Victim - Least Valuable Attacker heuristic.
    fn _get_move_mvv_lva(&mut self, m: chess::Move, is_following_pv_line: bool) -> u32 {
        if is_following_pv_line {
            if m == self.pv_table[0][self.result.ply as usize] {
                return 20000;
            }
        }

        let value = match m.capture {
            Some(c) => _MVV_LVA[m.piece as usize][c as usize],
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

    fn _order_moves(&mut self, moves: &mut Vec<chess::Move>, is_following_pv_line: bool) {
        moves.sort_by(|a, b| {
            let a_value = self._get_move_mvv_lva(*a, is_following_pv_line);
            let b_value = self._get_move_mvv_lva(*b, is_following_pv_line);
            return b_value.cmp(&a_value);
        });
    }

    fn _get_piece_value(&mut self, piece: chess::Piece, square: usize) -> i32 {
        match piece {
            chess::Piece::WhitePawn => {
                let mut score = 100;
                score += pst::PAWN_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteKnight => {
                let mut score = 300;
                score += pst::KNIGHT_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteBishop => {
                let mut score = 350;
                score += pst::BISHOP_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteRook => {
                let mut score = 500;
                score += pst::ROOK_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteQueen => {
                let mut score = 1000;
                score += pst::ROOK_POSITIONAL_SCORE[square] + pst::BISHOP_POSITIONAL_SCORE[square];
                return score;
            }
            chess::Piece::WhiteKing => {
                let mut score = 10000;
                score += pst::KING_POSITIONAL_SCORE[square];
                return score;
            }

            chess::Piece::BlackPawn => {
                let mut score = -100;
                score -= pst::PAWN_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackKnight => {
                let mut score = -300;
                score -= pst::KNIGHT_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackBishop => {
                let mut score = -350;
                score -= pst::BISHOP_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackRook => {
                let mut score = -500;
                score -= pst::ROOK_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackQueen => {
                let mut score = -1000;
                score -= pst::ROOK_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize]
                    + pst::BISHOP_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize];
                return score;
            }
            chess::Piece::BlackKing => {
                let mut score = -10000;
                score -= pst::KING_POSITIONAL_SCORE[pst::MIRROR_SCORE[square] as usize];
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
                score += self._get_piece_value(piece, square as usize);
            }
        }

        return if position.turn == chess::Color::White {
            score
        } else {
            -score
        };
    }
}
