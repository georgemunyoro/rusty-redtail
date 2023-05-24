use std::{
    collections::BinaryHeap,
    sync::{Arc, Mutex},
};

use crate::{
    board::{Board, Position},
    chess::{
        self,
        _move::{BitPackedMove, PrioritizedMove},
        color::Color,
        piece::Piece,
    },
    movegen::MoveGenerator,
    search::constants::*,
    search::options::*,
    tt::{self, TranspositionTable},
    utils, Cutoffs,
};

#[derive(Debug)]
pub struct PositionEvaluation {
    pub score: i32,
    pub best_move: Option<chess::_move::BitPackedMove>,
    pub depth: u8,
    pub ply: u32,
    pub nodes: i32,
    pub cutoffs: Cutoffs,
}

pub struct Evaluator {
    pub running: Arc<Mutex<bool>>,
    pub result: PositionEvaluation,
    pub killer_moves: [[chess::_move::BitPackedMove; MAX_PLY]; 2],
    pub history_moves: [[u32; MAX_PLY]; 12],
    pub started_at: u128,
    pub options: SearchOptions,
    pub tt: Arc<Mutex<TranspositionTable>>,
    pub repetition_table: Vec<u64>,
    counter_move_table: [[BitPackedMove; 64]; 64],
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
                cutoffs: Cutoffs::new(),
            },
            killer_moves: [[chess::_move::BitPackedMove::default(); MAX_PLY]; 2],
            history_moves: [[0; MAX_PLY]; 12],
            started_at: 0,
            options: SearchOptions::new(),
            tt: Arc::new(Mutex::new(TranspositionTable::new(1024))),
            repetition_table: Vec::with_capacity(150),
            counter_move_table: [[BitPackedMove::default(); 64]; 64],
        };
    }

    /// Determines the time to spend on a move based on the time left for the side to move
    /// as well as the increment.
    fn set_move_time(&mut self, position: &mut Position) {
        if self.options.movetime.is_some() || self.options.infinite {
            return;
        }
        let time_left_for_side = if position.turn == Color::White {
            match self.options.wtime {
                Some(wtime) => wtime,
                None => 0,
            }
        } else {
            match self.options.btime {
                Some(btime) => btime,
                None => 0,
            }
        };
        let increment = if position.turn == Color::White {
            match self.options.winc {
                Some(winc) => winc,
                None => 0,
            }
        } else {
            match self.options.binc {
                Some(binc) => binc,
                None => 0,
            }
        };

        let time_for_move = time_left_for_side / 45 + (increment / 2);

        if time_for_move >= time_left_for_side {
            self.options.movetime = Some(time_left_for_side - 500);
        } else {
            if time_for_move <= 0 {
                self.options.movetime = Some(200);
                return;
            }
            self.options.movetime = Some(time_for_move);
        }
    }

    pub fn get_best_move(
        &mut self,
        position: &mut Position,
        options: SearchOptions,
        running: Arc<Mutex<bool>>,
        transposition_table: Arc<Mutex<TranspositionTable>>,
        thread_id: usize,
        start_depth: u8,
    ) -> Option<chess::_move::BitPackedMove> {
        self.result = PositionEvaluation {
            score: 0,
            best_move: None,
            depth: 0,
            ply: 0,
            nodes: 0,
            cutoffs: Cutoffs::new(),
        };

        let depth = match options.depth {
            Some(depth) => depth as u8,
            None => MAX_PLY as u8,
        };

        self.options = options;
        self.set_move_time(position);

        self.running = running;
        self.tt = transposition_table;
        self.started_at = Evaluator::_get_time_ms();

        let mut alpha = -50000;
        let mut beta = 50000;
        let mut current_depth = start_depth;
        let mut pv_line_completed_so_far = Vec::new();
        self.repetition_table.clear();

        self.tt.lock().unwrap().increment_age();

        loop {
            if current_depth > depth {
                break;
            }

            let start_time = Evaluator::_get_time_ms();

            let score = self.negamax(position, alpha, beta, current_depth, false, None);

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

            if thread_id == 0 {
                self.print_info(position, start_time);
            }
            current_depth += 1;
        }

        if thread_id != 0 {
            return None;
        }

        let mut b = chess::_move::BitPackedMove::default();

        if pv_line_completed_so_far.len() > 0 {
            b = pv_line_completed_so_far[0].get_move();
            println!("bestmove {}", pv_line_completed_so_far[0].get_move());
        } else {
            println!("bestmove {}", chess::_move::BitPackedMove::default());
        }

        return Some(b);
    }

    pub fn negamax(
        &mut self,
        position: &mut Position,
        _alpha: i32,
        beta: i32,
        _depth: u8,
        was_last_move_null: bool,
        last_move: Option<BitPackedMove>,
    ) -> i32 {
        let mut alpha = _alpha;
        let mut depth = _depth; // will be mutable later for search extensions
        let mut alpha_move = chess::_move::BitPackedMove::default();

        if self.result.nodes & 2047 == 0 {
            self.set_running(self.check_time());
        }

        let is_in_check = position.is_in_check();

        if !is_in_check && depth > 2 && !was_last_move_null {
            position.make_null_move();
            self.result.ply += 1;

            let null_move_score = -self.negamax(position, -beta, -beta + 1, depth - 3, true, None);

            position.unmake_move();
            self.result.ply -= 1;

            if null_move_score >= beta {
                return beta;
            }
        }

        if self.result.ply >= MAX_PLY as u32 {
            return self.evaluate(position);
        }

        if is_in_check {
            depth += 1;
        }

        if self
            .repetition_table
            .iter()
            .filter(|&&x| x == position.hash)
            .count()
            >= 2
        {
            return -15;
        }

        self.result.nodes += 1;

        let tt_entry = self
            .tt
            .lock()
            .unwrap()
            .probe_entry(position.hash, depth, alpha, beta);

        if tt_entry.is_valid() {
            if tt_entry.get_flag() == tt::TranspositionTableEntryFlag::EXACT && self.result.ply == 0
            {
                self.result.depth = depth;
                self.result.score = tt_entry.get_value();
            }
            return tt_entry.get_value();
        }

        if depth == 0 {
            return self.quiescence(position, alpha, beta);
        }

        let mut legal_moves_searched = 0;
        let mut queue = self.order_moves_p(position.generate_moves(false), position, last_move);
        let mut found_pv = false;
        let mut hash_f = tt::TranspositionTableEntryFlag::ALPHA;

        while let Some(pm) = queue.pop() {
            let is_legal_move = position.make_move(pm.m, false);
            if !is_legal_move {
                continue;
            }

            self.result.ply += 1;
            self.repetition_table.push(position.hash);

            let mut _score = 0;

            // If we have found a pv move, then we need to search it with a null window
            if found_pv {
                _score = -self.negamax(position, -alpha - 1, -alpha, depth - 1, false, None);

                // If our pv move fails, then we need to search again with a full window
                if (_score > alpha) && (_score < beta) {
                    _score = -self.negamax(position, -beta, -alpha, depth - 1, false, None);
                }
            } else {
                if legal_moves_searched == 0 {
                    // If we have not found a pv move, and this is the first move, then we need to search with a full window
                    _score = -self.negamax(position, -beta, -alpha, depth - 1, false, None);
                } else {
                    if legal_moves_searched >= _FULL_DEPTH_MOVES
                        && depth >= _REDUCTION_LIMIT
                        && !position.is_in_check()
                        && !pm.m.is_promotion()
                        && !pm.m.is_capture()
                    {
                        _score =
                            -self.negamax(position, -alpha - 1, -alpha, depth - 2, false, None);
                    } else {
                        _score = alpha + 1;
                    }

                    // If we found a better move during LMR
                    if _score > alpha {
                        _score =
                            -self.negamax(position, -alpha - 1, -alpha, depth - 1, false, None);

                        // if our LMR move fails, then we need to search again with a full window
                        if (_score > alpha) && (_score < beta) {
                            _score = -self.negamax(position, -beta, -alpha, depth - 1, false, None);
                        }
                    }
                }
            }

            self.result.ply -= 1;
            self.repetition_table.pop();
            position.unmake_move();

            if !self.is_running() {
                return 0;
            }

            legal_moves_searched += 1;

            if _score >= beta {
                self.tt.lock().unwrap().save(
                    position.hash,
                    depth,
                    tt::TranspositionTableEntryFlag::BETA,
                    beta,
                    pm.m,
                );

                if self.killer_moves[0][self.result.ply as usize] != pm.m {
                    self.killer_moves[1][self.result.ply as usize] =
                        self.killer_moves[0][self.result.ply as usize];
                    self.killer_moves[0][self.result.ply as usize] = pm.m;
                }

                self.counter_move_table[pm.m.get_from() as usize][pm.m.get_to() as usize] = pm.m;

                // ==============
                // cutof testing
                // ==============

                if legal_moves_searched == 1 {
                    self.result.cutoffs.move_1 += 1;
                }
                if legal_moves_searched == 2 {
                    self.result.cutoffs.move_2 += 1;
                }
                self.result.cutoffs.avg_cutoff_move_no += legal_moves_searched as f32;
                self.result.cutoffs.total += 1;

                // ==============
                // ==============

                return beta;
            }

            if _score > alpha {
                hash_f = tt::TranspositionTableEntryFlag::EXACT;
                alpha_move = pm.m;
                found_pv = true;
                alpha = _score;

                if self.result.ply == 0 {
                    self.result.depth = depth;
                    self.result.score = _score;
                    self.result.best_move = Some(pm.m);
                }

                self.history_moves[pm.m.get_piece() as usize][self.result.ply as usize] +=
                    (depth * depth) as u32;
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

    fn quiescence(&mut self, position: &mut Position, _alpha: i32, beta: i32) -> i32 {
        let mut alpha = _alpha;

        if self.result.nodes & 2047 == 0 {
            self.set_running(self.check_time());
        }

        self.result.nodes += 1;

        let stand_pat = self.evaluate(position);
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat
        }

        let mut queue = self.order_moves_p(position.generate_moves(true), position, None);
        while let Some(pm) = queue.pop() {
            if !pm.m.is_capture() {
                continue;
            }

            let is_legal_capture = position.make_move(pm.m, true);
            if !is_legal_capture {
                continue;
            }

            self.result.ply += 1;
            let score = -self.quiescence(position, -beta, -alpha);
            self.result.ply -= 1;

            position.unmake_move();

            if !self.is_running() {
                return 0;
            }

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        return alpha;
    }

    pub fn print_info(&self, position: &mut Position, start_time: u128) {
        let stop_time: u128 = Evaluator::_get_time_ms();
        let nps: i32 =
            (self.result.nodes as f64 / ((stop_time - start_time) as f64 / 1000.0)) as i32;
        let pv_line_found: Vec<tt::TranspositionTableEntry> =
            self.tt.lock().unwrap().get_pv_line(position);

        let score = pv_line_found[0].get_value();

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
                "info score {} {} depth {} nodes {} nps {} time {} hashfull {}",
                if is_mate { "mate" } else { "cp" },
                if is_mate { mate_in } else { score },
                self.result.depth,
                self.result.nodes,
                nps,
                stop_time - start_time,
                self.tt.lock().unwrap().get_hashfull()
            );

            let mut pv_str: String = String::new();

            for i in pv_line_found {
                pv_str.push_str(" ");
                pv_str.push_str(i.get_move().to_string().as_str());
            }

            println!(" info pv{}", pv_str);
        }
    }

    fn check_time(&self) -> bool {
        if self.options.infinite {
            return true;
        }

        let elapsed: u128 = Evaluator::_get_time_ms() - self.started_at;

        match self.options.movetime {
            Some(movetime) => {
                if (elapsed + 200) >= movetime as u128 {
                    return false;
                }
            }
            None => {}
        }

        return true;
    }

    fn _has_non_pawn_material(&self, position: &mut Position) -> bool {
        if position.turn == Color::White {
            (position.bitboards[Piece::WhiteBishop as usize]
                + position.bitboards[Piece::WhiteKnight as usize]
                + position.bitboards[Piece::WhiteRook as usize]
                + position.bitboards[Piece::WhiteQueen as usize])
                != 0
        } else {
            (position.bitboards[Piece::BlackBishop as usize]
                + position.bitboards[Piece::BlackKnight as usize]
                + position.bitboards[Piece::BlackRook as usize]
                + position.bitboards[Piece::BlackQueen as usize])
                != 0
        }
    }

    pub fn _get_time_ms() -> u128 {
        let now: std::time::SystemTime = std::time::SystemTime::now();
        let since_the_epoch: std::time::Duration = now
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        return since_the_epoch.as_millis();
    }

    /// Returns a score for a move based on various heuristics.
    fn get_move_priority(
        &mut self,
        m: chess::_move::BitPackedMove,
        is_following_pv_line: bool,
        last_move: Option<BitPackedMove>,
    ) -> u32 {
        if let Some(m) = last_move {
            let counter_move = self.counter_move_table[m.get_from() as usize][m.get_to() as usize];
            if counter_move != BitPackedMove::default() {
                return 30000;
            }
        }

        if is_following_pv_line {
            return 20000;
        }

        if m.is_capture() {
            return _MVV_LVA[m.get_piece() as usize][m.get_capture() as usize] + 1000;
        }

        if self.killer_moves[0][self.result.ply as usize] == m {
            return 9001;
        }

        if self.killer_moves[1][self.result.ply as usize] == m {
            return 9000;
        }

        return self.history_moves[m.get_piece() as usize][self.result.ply as usize] as u32;
    }

    fn order_moves_p(
        &mut self,
        moves: Vec<chess::_move::BitPackedMove>,
        position: &mut Position,
        last_move: Option<BitPackedMove>,
    ) -> BinaryHeap<PrioritizedMove> {
        let mut queue: BinaryHeap<PrioritizedMove> = BinaryHeap::with_capacity(moves.len());

        let mut tt_move: chess::_move::BitPackedMove = chess::_move::BitPackedMove::default();
        if let Some(tt_entry) = self.tt.lock().unwrap().get(position.hash) {
            if tt_entry.get_move() != chess::_move::BitPackedMove::default()
                && tt_entry.get_flag() == tt::TranspositionTableEntryFlag::EXACT
            {
                tt_move = tt_entry.get_move();
            }
        }

        for m in moves {
            let priority: u32 = self.get_move_priority(m, m == tt_move, last_move);
            queue.push(PrioritizedMove { m, priority })
        }

        queue
    }

    pub fn evaluate(&mut self, position: &mut Position) -> i32 {
        let mut score = position.material[position.turn as usize]
            - position.material[(!position.turn) as usize];

        let mut white_score = 0;
        let mut black_score = 0;

        let mut white_pawns = position.bitboards[Piece::WhitePawn as usize];
        while white_pawns != 0 {
            let square = utils::pop_lsb(&mut white_pawns);
            let doubled_pawns = utils::count_bits(
                position.bitboards[Piece::WhitePawn as usize]
                    & position.file_masks[square as usize],
            );
            if doubled_pawns > 1 {
                white_score += DOUBLED_PAWN_PENALTY as i32 * doubled_pawns as i32;
            }

            if (position.bitboards[Piece::WhitePawn as usize]
                & position.isolated_pawn_masks[square as usize])
                == 0
            {
                white_score += ISOLATED_PAWN_PENALTY as i32;
            }

            if (position.white_passed_pawn_masks[square as usize]
                & position.bitboards[Piece::BlackPawn as usize])
                == 0
            {
                white_score += PASSED_PAWN_BONUS[GET_RANK[square as usize] as usize] as i32
            }
        }

        let mut black_pawns = position.bitboards[Piece::BlackPawn as usize];
        while black_pawns != 0 {
            let square = utils::pop_lsb(&mut black_pawns);
            let doubled_pawns = utils::count_bits(
                position.bitboards[Piece::BlackPawn as usize]
                    & position.file_masks[square as usize],
            );
            if doubled_pawns > 1 {
                black_score += DOUBLED_PAWN_PENALTY as i32 * doubled_pawns as i32;
            }

            if (position.bitboards[Piece::BlackPawn as usize]
                & position.isolated_pawn_masks[square as usize])
                == 0
            {
                black_score += ISOLATED_PAWN_PENALTY as i32;
            }

            if (position.black_passed_pawn_masks[square as usize]
                & position.bitboards[Piece::WhitePawn as usize])
                == 0
            {
                black_score += PASSED_PAWN_BONUS[7 - GET_RANK[square as usize] as usize] as i32
            }
        }

        let mut white_rooks = position.bitboards[Piece::WhiteRook as usize];
        while white_rooks != 0 {
            let square = utils::pop_lsb(&mut white_rooks);

            // Semi open files
            if ((position.bitboards[Piece::WhitePawn as usize])
                & position.file_masks[square as usize])
                == 0
            {
                white_score += SEMI_OPEN_FILE_SCORE;
            }

            // Open files
            if ((position.bitboards[Piece::WhitePawn as usize]
                | position.bitboards[Piece::BlackPawn as usize])
                & position.file_masks[square as usize])
                == 0
            {
                white_score += OPEN_FILE_SCORE;
            }
        }

        let mut black_rooks = position.bitboards[Piece::BlackRook as usize];
        while black_rooks != 0 {
            let square = utils::pop_lsb(&mut black_rooks);

            // Semi open files
            if ((position.bitboards[Piece::BlackPawn as usize])
                & position.file_masks[square as usize])
                == 0
            {
                black_score += SEMI_OPEN_FILE_SCORE;
            }

            // Open files
            if ((position.bitboards[Piece::WhitePawn as usize]
                | position.bitboards[Piece::BlackPawn as usize])
                & position.file_masks[square as usize])
                == 0
            {
                black_score += OPEN_FILE_SCORE;
            }
        }

        // white king safety
        let white_king_position = utils::get_lsb(position.bitboards[Piece::WhiteKing as usize]);
        white_score += utils::count_bits(
            position.king_attacks[white_king_position as usize]
                & position.bitboards[Piece::WhitePawn as usize],
        ) as i32
            * 6;

        // black king safety
        let black_king_position = utils::get_lsb(position.bitboards[Piece::BlackKing as usize]);
        black_score += utils::count_bits(
            position.king_attacks[black_king_position as usize]
                & position.bitboards[Piece::BlackPawn as usize],
        ) as i32
            * 6;

        if position.turn == Color::White {
            score += white_score;
            score -= black_score;
        } else {
            score += black_score;
            score -= white_score;
        }

        return score;
    }
}
