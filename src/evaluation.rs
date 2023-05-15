use std::{
    collections::BinaryHeap,
    ptr::null,
    sync::{Arc, Mutex},
};

use crate::{
    board::{self, Board, Position},
    chess::{self, PrioritizedMove},
    movegen::MoveGenerator,
    pst,
    tt::{self, TranspositionTable},
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
            // movetime: Some(3000),
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
    pub started_at: u128,
    pub options: SearchOptions,
    pub tt: Arc<Mutex<TranspositionTable>>,
    pub repetition_table: Vec<u64>,
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
            started_at: 0,
            options: SearchOptions::new(),
            tt: Arc::new(Mutex::new(TranspositionTable::new())),
            repetition_table: Vec::with_capacity(150),
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

        let mut alpha = -50000;
        let mut beta = 50000;
        let mut current_depth = 1;

        let mut pv_line_completed_so_far = Vec::new();

        loop {
            if current_depth > depth {
                break;
            }

            let start_time = self._get_time_ms();

            let score = self.negamax(position, alpha, beta, current_depth, false);

            if score <= alpha || score >= beta {
                alpha = -50000;
                beta = 50000;
                continue;
            }

            alpha = score - 50;
            beta = score + 50;

            let pv_line_found = self.tt.lock().unwrap().get_pv_line(position);
            if pv_line_found.len() > 0 {
                pv_line_completed_so_far = pv_line_found;
            }

            if !self.is_running() {
                break;
            }

            self.print_info(position, start_time);
            current_depth += 1;
        }

        let mut b = chess::NULL_MOVE;

        if pv_line_completed_so_far.len() > 0 {
            b = pv_line_completed_so_far[0].m.unwrap();
            println!("bestmove {}", pv_line_completed_so_far[0].m.unwrap());
        } else {
            println!("bestmove {}", chess::NULL_MOVE);
        }

        return Some(b);
    }

    fn negamax(
        &mut self,
        position: &mut Position,
        _alpha: i32,
        beta: i32,
        _depth: u8,
        was_last_move_null: bool,
    ) -> i32 {
        let mut alpha = _alpha;
        let depth = _depth; // will be mutable later for search extensions
        let mut alpha_move = None;

        if self.result.nodes & 2047 == 0 {
            self.set_running(self.check_time());
        }

        let is_in_check = position.is_in_check();

        if !is_in_check && depth > 2 && !was_last_move_null {
            position.make_null_move();
            self.result.ply += 1;

            let null_move_score = -self.negamax(position, -beta, -beta + 1, depth - 3, true);

            position.unmake_move();
            self.result.ply -= 1;

            if null_move_score >= beta {
                return beta;
            }
        }

        if self
            .repetition_table
            .iter()
            .filter(|&&x| x == position.hash)
            .count()
            >= 2
        {
            return 0;
        }

        self.result.nodes += 1;

        let mut hash_f = tt::TranspositionTableEntryFlag::ALPHA;

        if let Some(tt_value) =
            self.tt
                .lock()
                .unwrap()
                .probe_entry(position.hash, depth, alpha, beta)
        {
            if tt_value.1 == tt::TranspositionTableEntryFlag::EXACT {
                if self.result.ply == 0 {
                    self.result.depth = depth;
                    self.result.score = tt_value.0;
                }
            }
            return tt_value.0;
        }

        if depth == 0 {
            let evaluation = self.evaluate(position);
            return evaluation;
        }

        let mut legal_moves_searched = 0;

        let moves = position.generate_moves();
        let mut queue = self.order_moves_p(moves, position);

        let mut found_pv = false;

        while let Some(pm) = queue.pop() {
            let is_legal_move = position.make_move(pm.m, false);
            if !is_legal_move {
                continue;
            }

            legal_moves_searched += 1;
            self.result.ply += 1;
            self.repetition_table.push(position.hash);

            let score = if found_pv {
                let pvs_score = -self.negamax(position, -alpha - 1, -alpha, depth - 1, false);
                if (pvs_score > alpha) && (pvs_score < beta) {
                    -self.negamax(position, -beta, -alpha, depth - 1, false)
                } else {
                    pvs_score
                }
            } else {
                -self.negamax(position, -beta, -alpha, depth - 1, false)
            };

            self.result.ply -= 1;
            self.repetition_table.pop();
            position.unmake_move();

            if !self.is_running() {
                return 0;
            }

            if score >= beta {
                self.tt.lock().unwrap().save(
                    position.hash,
                    depth,
                    tt::TranspositionTableEntryFlag::BETA,
                    beta,
                    Some(pm.m),
                );

                if self.killer_moves[0][self.result.ply as usize] != pm.m {
                    self.killer_moves[1][self.result.ply as usize] =
                        self.killer_moves[0][self.result.ply as usize];
                    self.killer_moves[0][self.result.ply as usize] = pm.m;
                }

                return beta;
            }

            if score > alpha {
                hash_f = tt::TranspositionTableEntryFlag::EXACT;
                alpha_move = Some(pm.m);
                found_pv = true;
                alpha = score;

                if self.result.ply == 0 {
                    self.result.depth = depth;
                    self.result.score = score;
                    self.result.best_move = Some(pm.m);
                }
            }
        }

        if legal_moves_searched == 0 {
            if is_in_check {
                alpha = -49000 + self.result.ply as i32;
            } else {
                alpha = 0;
            }
        }

        self.tt
            .lock()
            .unwrap()
            .save(position.hash, depth, hash_f, alpha, alpha_move);

        return alpha;
    }

    fn print_info(&self, position: &mut Position, start_time: u128) {
        let stop_time = self._get_time_ms();
        let nps = (self.result.nodes as f64 / ((stop_time - start_time) as f64 / 1000.0)) as i32;
        let pv_line_found = self.tt.lock().unwrap().get_pv_line(position);

        let score = pv_line_found[0].value;

        if pv_line_found.len() > 0 {
            let is_mate = score > 48000;
            let mut mate_in: i32 = 0;

            if is_mate {
                let x = -(score - 49000);
                if x % 2 == 0 {
                    mate_in = x as i32 / 2;
                } else {
                    mate_in = (x as i32 + 1) / 2;
                }
            }

            print!(
                "info score {} {} depth {} nodes {} nps {} time {}",
                if is_mate { "mate" } else { "cp" },
                if is_mate { mate_in } else { score },
                self.result.depth,
                self.result.nodes,
                nps,
                stop_time - start_time
            );

            let mut pv_str = String::new();

            for i in pv_line_found {
                pv_str.push_str(" ");
                pv_str.push_str(i.m.unwrap().to_string().as_str());
            }

            println!(" info pv{}", pv_str);
        }
    }

    fn check_time(&self) -> bool {
        if self.options.infinite {
            return true;
        }

        let elapsed = self._get_time_ms() - self.started_at;

        match self.options.movetime {
            Some(movetime) => {
                if (elapsed + 100) >= movetime as u128 {
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

    /// Returns a score for a move based on various heuristics.
    fn get_move_priority(&mut self, m: chess::Move, is_following_pv_line: bool) -> u32 {
        if is_following_pv_line {
            return 20000;
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

                return 0;
            }
        };

        return value;
    }

    fn order_moves_p(
        &mut self,
        moves: Vec<chess::Move>,
        position: &mut Position,
    ) -> BinaryHeap<chess::PrioritizedMove> {
        let mut queue = BinaryHeap::new();

        let mut tt_move = chess::NULL_MOVE;
        if let Some(tt_entry) = self.tt.lock().unwrap().get(position.hash) {
            if let Some(m) = tt_entry.m {
                tt_move = m;
            }
        }

        for m in moves {
            let priority = self.get_move_priority(m, m == tt_move);
            queue.push(PrioritizedMove { m, priority })
        }

        queue
    }

    pub fn evaluate(&mut self, position: &mut Position) -> i32 {
        return position.material[position.turn as usize]
            - position.material[(!position.turn) as usize];
    }
}
