use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use crate::{
    board::{Board, Position},
    chess::{
        self,
        _move::BitPackedMove,
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

/// Triangular PV table for storing principal variation during search
pub struct PVTable {
    /// pv_table[ply] contains the PV starting from that ply
    table: [[BitPackedMove; MAX_PLY]; MAX_PLY],
    /// Length of PV at each ply
    length: [usize; MAX_PLY],
}

impl PVTable {
    pub fn new() -> Self {
        PVTable {
            table: [[BitPackedMove::default(); MAX_PLY]; MAX_PLY],
            length: [0; MAX_PLY],
        }
    }

    /// Clear the PV length at the given ply (called at start of node)
    pub fn clear_at(&mut self, ply: usize) {
        if ply < MAX_PLY {
            self.length[ply] = 0;
        }
    }

    /// Update PV when a new best move is found
    /// Copies the PV from ply+1 and prepends the current move
    pub fn update(&mut self, ply: usize, m: BitPackedMove) {
        if ply >= MAX_PLY {
            return;
        }
        self.table[ply][0] = m;
        if ply + 1 < MAX_PLY {
            let child_len = self.length[ply + 1];
            for i in 0..child_len {
                if i + 1 < MAX_PLY {
                    self.table[ply][i + 1] = self.table[ply + 1][i];
                }
            }
            self.length[ply] = child_len + 1;
        } else {
            self.length[ply] = 1;
        }
    }

    /// Get the PV from ply 0 as a vector of moves
    pub fn get_pv(&self) -> Vec<BitPackedMove> {
        let len = self.length[0];
        self.table[0][..len].to_vec()
    }
}

pub struct Evaluator {
    pub running: bool,
    pub result: PositionEvaluation,
    pub killer_moves: [[chess::_move::BitPackedMove; MAX_PLY]; 2],
    /// History heuristic table: indexed by [piece][to_square]
    pub history_moves: [[i32; 64]; 12],
    pub started_at: u128,
    pub options: SearchOptions,
    pub repetition_table: Vec<u64>,
    counter_move_table: [[BitPackedMove; 64]; 64],
    stop_flag: Option<Arc<AtomicBool>>,
    pv_table: PVTable,
    silent: bool,
    /// Soft time limit: don't start a new iteration if elapsed exceeds this
    optimum_time: u32,
    /// Hard time limit: abort the search immediately if elapsed approaches this
    maximum_time: u32,
    /// Shared flag for pondering: true while pondering, set to false on ponderhit
    ponder_flag: Option<Arc<AtomicBool>>,
    /// Side to move at search start, cached for ponderhit time recalc
    search_turn: Color,
    /// Depth offset for Lazy SMP thread diversity (odd helper threads start at depth+1)
    depth_offset: u8,
    /// Whether this is the main search thread (thread 0) — controls UCI output and stop signaling
    is_main_thread: bool,
    /// Precomputed LMR reduction table: [depth][move_count]
    lmr_table: [[u8; 64]; 64],
}

impl Evaluator {
    pub fn new() -> Evaluator {
        let mut lmr_table = [[0u8; 64]; 64];
        for depth in 1..64 {
            for moves in 1..64 {
                lmr_table[depth][moves] =
                    (1.0 + (depth as f64).ln() * (moves as f64).ln() / 2.00).max(1.0) as u8;
            }
        }

        Evaluator {
            running: false,
            result: PositionEvaluation {
                score: 0,
                best_move: None,
                depth: 0,
                ply: 0,
                nodes: 0,
                cutoffs: Cutoffs::new(),
            },
            killer_moves: [[chess::_move::BitPackedMove::default(); MAX_PLY]; 2],
            history_moves: [[0; 64]; 12],
            started_at: 0,
            options: SearchOptions::new(),
            repetition_table: Vec::with_capacity(150),
            counter_move_table: [[BitPackedMove::default(); 64]; 64],
            stop_flag: None,
            pv_table: PVTable::new(),
            silent: false,
            optimum_time: 0,
            maximum_time: 0,
            ponder_flag: None,
            search_turn: Color::White,
            depth_offset: 0,
            is_main_thread: true,
            lmr_table,
        }
    }

    pub fn set_silent(&mut self, silent: bool) {
        self.silent = silent;
    }

    fn is_stopped(&self) -> bool {
        if let Some(ref flag) = self.stop_flag {
            flag.load(Ordering::SeqCst)
        } else {
            false
        }
    }

    /// Computes optimum (soft) and maximum (hard) time limits for a move,
    /// using a Stockfish-inspired algorithm.
    fn set_move_time(&mut self, turn: Color) {
        if self.options.movetime.is_some() {
            self.optimum_time = self.options.movetime.unwrap();
            self.maximum_time = self.options.movetime.unwrap();
            return;
        }
        if self.options.infinite || self.options.ponder {
            self.optimum_time = u32::MAX;
            self.maximum_time = u32::MAX;
            return;
        }

        let time_left = if turn == Color::White {
            self.options.wtime.unwrap_or(0)
        } else {
            self.options.btime.unwrap_or(0)
        };
        let increment = if turn == Color::White {
            self.options.winc.unwrap_or(0)
        } else {
            self.options.binc.unwrap_or(0)
        };

        let moves = self.options.movestogo.unwrap_or(25).max(1);
        let base = time_left / moves + increment / 2;
        let hard_cap = time_left / 3;

        self.optimum_time = base.min(hard_cap).max(1);
        self.maximum_time = (base * 3).min(hard_cap).max(1);
    }

    pub fn get_best_move(
        &mut self,
        position: &mut Position,
        options: SearchOptions,
        tt: &TranspositionTable,
        stop_flag: &Arc<AtomicBool>,
        ponder_flag: Option<Arc<AtomicBool>>,
    ) -> (Option<chess::_move::BitPackedMove>, Option<chess::_move::BitPackedMove>) {
        self.stop_flag = Some(Arc::clone(stop_flag));
        self.ponder_flag = ponder_flag;
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
        self.search_turn = position.turn;
        self.set_move_time(position.turn);

        self.running = true;
        self.started_at = Evaluator::_get_time_ms();

        let mut alpha = -50000;
        let mut beta = 50000;
        let mut current_depth = 1 + self.depth_offset;
        let mut pv_completed_so_far: Vec<BitPackedMove> = Vec::new();
        self.repetition_table.clear();
        self.pv_table = PVTable::new();

        // Get a fallback move in case search doesn't complete
        let legal_moves = position.generate_moves(false);
        let mut fallback_move = chess::_move::BitPackedMove::default();
        for m in legal_moves {
            if position.make_move(m, false) {
                position.unmake_move();
                fallback_move = m;
                break;
            }
        }

        loop {
            if current_depth > depth {
                break;
            }

            let start_time = Evaluator::_get_time_ms();

            let score = self.negamax(position, alpha, beta, current_depth, false, None, tt);

            // If score falls outside aspiration window and we haven't already widened it
            if (score <= alpha || score >= beta) && !(alpha == -50000 && beta == 50000) {
                // Widen the window and re-search at same depth
                alpha = -50000;
                beta = 50000;
                continue;
            }

            alpha = score - 50;
            beta = score + 50;

            if !self.running {
                break;
            }

            // Save PV only from fully completed iterations
            let pv = self.pv_table.get_pv();
            if !pv.is_empty() {
                pv_completed_so_far = pv;
            }

            self.print_info(start_time, score, &pv_completed_so_far, tt);
            current_depth += 1;

            // Soft time check: don't start the next iteration if we've used
            // more than half the optimum time — it's unlikely to complete.
            if self.elapsed_ms() > self.optimum_time / 2 {
                break;
            }
        }

        let best_move = if !pv_completed_so_far.is_empty() {
            pv_completed_so_far[0]
        } else {
            fallback_move
        };

        // Verify the best move is legal before outputting
        let final_move = if position.make_move(best_move, false) {
            position.unmake_move();
            best_move
        } else {
            // If best move is illegal, fall back to first legal move
            fallback_move
        };

        // Check if the 2nd PV move is a legal ponder move
        let ponder_move = if pv_completed_so_far.len() >= 2 {
            let candidate = pv_completed_so_far[1];
            if position.make_move(final_move, false) {
                let legal = position.make_move(candidate, false);
                if legal {
                    position.unmake_move();
                }
                position.unmake_move();
                if legal { Some(candidate) } else { None }
            } else {
                None
            }
        } else {
            None
        };

        // Main thread signals helpers to stop
        if self.is_main_thread {
            if let Some(ref flag) = self.stop_flag {
                flag.store(true, Ordering::SeqCst);
            }
        }

        if !self.silent {
            if let Some(pm) = ponder_move {
                println!("bestmove {} ponder {}", final_move, pm);
            } else {
                println!("bestmove {}", final_move);
            }
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
        (Some(final_move), ponder_move)
    }

    pub fn negamax(
        &mut self,
        position: &mut Position,
        _alpha: i32,
        beta: i32,
        _depth: u8,
        was_last_move_null: bool,
        last_move: Option<BitPackedMove>,
        tt: &TranspositionTable,
    ) -> i32 {
        let mut alpha = _alpha;
        let mut depth = _depth;
        let mut alpha_move = chess::_move::BitPackedMove::default();
        let is_pv_node = beta - alpha > 1;

        // Prevent stack overflow from deep recursion
        if self.result.ply >= MAX_PLY as u32 - 1 {
            return self.evaluate(position);
        }

        if self.result.nodes & 2047 == 0 {
            self.running = self.check_time();
        }

        // Repetition detection (check early to avoid wasting time)
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

        // TT probe
        let tt_entry = tt.probe_entry(position.hash, depth, alpha, beta);

        if tt_entry.is_valid() && !is_pv_node {
            if tt_entry.get_flag() == tt::TranspositionTableEntryFlag::EXACT && self.result.ply == 0
            {
                self.result.depth = depth;
                self.result.score = tt_entry.get_value();
            }
            return tt_entry.get_value();
        }

        let is_in_check = position.is_in_check();

        // Check extension
        if is_in_check {
            depth += 1;
        }

        if depth == 0 {
            return self.quiescence(position, alpha, beta, tt);
        }

        // Static evaluation for pruning decisions (only when not in check)
        let static_eval = if !is_in_check {
            self.evaluate(position)
        } else {
            -50000 // Don't prune when in check
        };

        // === Reverse Futility Pruning (RFP) ===
        // If our position is so good that even with a margin, it beats beta,
        // the opponent would never allow this position. Prune.
        if !is_pv_node
            && !is_in_check
            && depth <= 7
            && static_eval - RFP_MARGIN * depth as i32 >= beta
            && static_eval < 40000 // Don't prune near mate scores
        {
            return static_eval;
        }

        // === Razoring ===
        // If eval is far below alpha at low depth, it's unlikely a quiet move
        // can raise it. Drop directly to quiescence search.
        if !is_pv_node
            && !is_in_check
            && depth <= 3
            && static_eval + RAZOR_MARGIN + RAZOR_MARGIN * (depth as i32 - 1) < alpha
        {
            let q_score = self.quiescence(position, alpha, beta, tt);
            if q_score < alpha {
                return q_score;
            }
        }

        // === Null Move Pruning ===
        // If our position is good enough that even passing the move still
        // causes a beta cutoff, prune the whole subtree.
        if !is_in_check
            && !is_pv_node
            && depth > 2
            && !was_last_move_null
            && self._has_non_pawn_material(position)
        {
            // Adaptive reduction: deeper searches get more reduction
            let r = 3 + depth as u32 / 6;

            position.make_null_move();
            self.result.ply += 1;

            let null_move_score = -self.negamax(
                position,
                -beta,
                -beta + 1,
                depth.saturating_sub(r as u8),
                true,
                None,
                tt,
            );

            position.unmake_move();
            self.result.ply -= 1;

            if !self.running {
                return 0;
            }

            if null_move_score >= beta {
                return beta;
            }
        }

        // === Internal Iterative Reduction (IIR) ===
        // If we don't have a TT move at this node, the move ordering will be
        // worse. Reduce depth by 1 to save time on these poorly-ordered nodes.
        let tt_move = self.get_tt_move(position, tt);
        if depth >= 4 && tt_move == BitPackedMove::default() && !is_in_check {
            depth -= 1;
        }

        // Move generation and scoring
        let raw_moves = position.generate_moves(false);
        let mut moves = self.score_moves(raw_moves, position, last_move, tt);

        let mut legal_moves_searched: u32 = 0;
        let mut found_pv = false;
        let mut hash_f = tt::TranspositionTableEntryFlag::ALPHA;

        // Clear PV at this ply
        self.pv_table.clear_at(self.result.ply as usize);

        let mut move_idx = 0;
        while move_idx < moves.len() {
            // Pick-best: find the highest-scored move from remaining
            let mut best = move_idx;
            for i in (move_idx + 1)..moves.len() {
                if moves[i].1 > moves[best].1 {
                    best = i;
                }
            }
            moves.swap(move_idx, best);
            let pm = moves[move_idx].0;
            move_idx += 1;

            let is_legal_move = position.make_move(pm, false);
            if !is_legal_move {
                continue;
            }

            self.result.ply += 1;
            self.repetition_table.push(position.hash);

            let is_quiet = !pm.is_capture() && !pm.is_promotion();

            // === Late Move Pruning (LMP) ===
            // At low depths, skip quiet moves that appear late in move ordering.
            // These are very unlikely to be the best move.
            if !is_pv_node
                && !is_in_check
                && is_quiet
                && depth <= 7
                && legal_moves_searched >= LMP_MOVE_COUNTS[depth as usize] as u32
            {
                self.result.ply -= 1;
                self.repetition_table.pop();
                position.unmake_move();
                continue;
            }

            // === SEE pruning for bad captures at low depths ===
            if !is_pv_node
                && !is_in_check
                && pm.is_capture()
                && depth <= 4
                && legal_moves_searched > 0
                && self.see(position, pm) < -(depth as i32 * 100)
            {
                self.result.ply -= 1;
                self.repetition_table.pop();
                position.unmake_move();
                continue;
            }

            let mut _score: i32;

            if found_pv {
                // PVS: search with null window first
                _score = -self.negamax(position, -alpha - 1, -alpha, depth - 1, false, Some(pm), tt);

                // Re-search with full window if it beats alpha
                if (_score > alpha) && (_score < beta) {
                    _score = -self.negamax(position, -beta, -alpha, depth - 1, false, Some(pm), tt);
                }
            } else if legal_moves_searched == 0 {
                // First move: full window search
                _score = -self.negamax(position, -beta, -alpha, depth - 1, false, Some(pm), tt);
            } else {
                // === Late Move Reduction (LMR) ===
                // Later moves in the move list are less likely to be good.
                // Search them at reduced depth first.
                let do_lmr = legal_moves_searched >= _FULL_DEPTH_MOVES as u32
                    && depth >= _REDUCTION_LIMIT
                    && !is_in_check
                    && is_quiet;

                if do_lmr {
                    // Use precomputed log-based reduction table
                    let mut r = self.lmr_table[depth.min(63) as usize]
                        [legal_moves_searched.min(63) as usize] as u8;

                    // Reduce more for non-PV nodes
                    if !is_pv_node {
                        r += 1;
                    }

                    // Don't reduce below 1
                    let reduced_depth = (depth - 1).saturating_sub(r).max(1);

                    _score = -self.negamax(
                        position,
                        -alpha - 1,
                        -alpha,
                        reduced_depth,
                        false,
                        Some(pm),
                        tt,
                    );
                } else {
                    // Force re-search at full depth
                    _score = alpha + 1;
                }

                // If LMR search found something interesting, re-search at full depth
                if _score > alpha {
                    _score = -self.negamax(
                        position,
                        -alpha - 1,
                        -alpha,
                        depth - 1,
                        false,
                        Some(pm),
                        tt,
                    );

                    // Full window re-search if null window failed
                    if (_score > alpha) && (_score < beta) {
                        _score = -self.negamax(
                            position, -beta, -alpha, depth - 1, false, Some(pm), tt,
                        );
                    }
                }
            }

            self.result.ply -= 1;
            self.repetition_table.pop();
            position.unmake_move();

            if !self.running {
                return 0;
            }

            legal_moves_searched += 1;

            if _score >= beta {
                tt.save(
                    position.hash,
                    depth,
                    tt::TranspositionTableEntryFlag::BETA,
                    beta,
                    pm,
                );

                // Update killer moves for quiet moves that cause beta cutoffs
                if is_quiet {
                    if self.killer_moves[0][self.result.ply as usize] != pm {
                        self.killer_moves[1][self.result.ply as usize] =
                            self.killer_moves[0][self.result.ply as usize];
                        self.killer_moves[0][self.result.ply as usize] = pm;
                    }

                    // Update history heuristic (by piece and destination square)
                    self.history_moves[pm.get_piece() as usize][pm.get_to() as usize] +=
                        (depth as i32) * (depth as i32);

                    // Penalize quiet moves that didn't cause a cutoff (history malus)
                    for i in 0..move_idx.saturating_sub(1) {
                        let prev = moves[i].0;
                        if !prev.is_capture() && !prev.is_promotion() {
                            self.history_moves[prev.get_piece() as usize]
                                [prev.get_to() as usize] -=
                                (depth as i32) * (depth as i32);
                        }
                    }
                }

                self.counter_move_table[pm.get_from() as usize][pm.get_to() as usize] = pm;

                // Cutoff tracking
                if legal_moves_searched == 1 {
                    self.result.cutoffs.move_1 += 1;
                }
                if legal_moves_searched == 2 {
                    self.result.cutoffs.move_2 += 1;
                }
                self.result.cutoffs.avg_cutoff_move_no += legal_moves_searched as f32;
                self.result.cutoffs.total += 1;

                return beta;
            }

            if _score > alpha {
                hash_f = tt::TranspositionTableEntryFlag::EXACT;
                alpha_move = pm;
                found_pv = true;
                alpha = _score;

                // Update PV table
                self.pv_table.update(self.result.ply as usize, pm);

                if self.result.ply == 0 {
                    self.result.depth = depth;
                    self.result.score = _score;
                    self.result.best_move = Some(pm);
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

        tt.save(position.hash, depth, hash_f, alpha, alpha_move);

        alpha
    }

    fn quiescence(
        &mut self,
        position: &mut Position,
        _alpha: i32,
        beta: i32,
        tt: &TranspositionTable,
    ) -> i32 {
        let mut alpha = _alpha;

        // Prevent stack overflow from deep recursion
        if self.result.ply >= MAX_PLY as u32 - 1 {
            return self.evaluate(position);
        }

        if self.result.nodes & 2047 == 0 {
            self.running = self.check_time();
        }

        self.result.nodes += 1;

        let stand_pat = self.evaluate(position);
        if stand_pat >= beta {
            return beta;
        }

        // === Delta Pruning ===
        // If even the best possible capture (queen value) can't raise our score
        // above alpha, don't bother searching captures.
        if stand_pat + SEE_PIECE_VALUES[Piece::WhiteQueen as usize] + DELTA_MARGIN < alpha {
            return alpha;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let raw_moves = position.generate_moves(true);
        let mut moves = self.score_moves(raw_moves, position, None, tt);

        let mut move_idx = 0;
        while move_idx < moves.len() {
            // Pick-best
            let mut best = move_idx;
            for i in (move_idx + 1)..moves.len() {
                if moves[i].1 > moves[best].1 {
                    best = i;
                }
            }
            moves.swap(move_idx, best);
            let pm = moves[move_idx].0;
            move_idx += 1;

            if !pm.is_capture() {
                continue;
            }

            // === Delta pruning per-move ===
            // If this specific capture can't raise alpha, skip it
            if stand_pat + SEE_PIECE_VALUES[pm.get_capture() as usize] + DELTA_MARGIN < alpha
                && !pm.is_promotion()
            {
                continue;
            }

            // === SEE pruning in quiescence ===
            // Don't search captures that lose material
            if self.see(position, pm) < 0 {
                continue;
            }

            let is_legal_capture = position.make_move(pm, true);
            if !is_legal_capture {
                continue;
            }

            self.result.ply += 1;
            let score = -self.quiescence(position, -beta, -alpha, tt);
            self.result.ply -= 1;

            position.unmake_move();

            if !self.running {
                return 0;
            }

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// Static Exchange Evaluation (SEE)
    /// Returns the material gain/loss of a capture sequence starting with move m.
    /// Positive means the capture wins material, negative means it loses.
    fn see(&self, position: &Position, m: BitPackedMove) -> i32 {
        let target = m.get_to();
        let mut gain: [i32; 32] = [0; 32];
        let mut d: usize = 0;

        let mut occ = position.occupancies[2];
        let mut attacker = m.get_piece();
        let mut side = position.turn;

        // Initial capture gain
        gain[0] = SEE_PIECE_VALUES[m.get_capture() as usize];

        // Remove the initial attacker from occupancy
        occ ^= 1u64 << (m.get_from() as u8);

        // Recompute all attackers with updated occupancy (handles x-ray discovery)
        let mut attackers = position.get_all_attackers_to(target, occ) & occ;

        side = !side;

        loop {
            d += 1;
            if d >= 32 {
                break;
            }

            // Speculative store: if opponent captures our piece on the target
            gain[d] = SEE_PIECE_VALUES[attacker as usize] - gain[d - 1];

            // Stand pat pruning: if neither side can improve, break
            if (-gain[d - 1]).max(gain[d]) < 0 {
                break;
            }

            // Find least valuable attacker of current side
            let side_offset = side as usize * 6;
            let mut found = false;
            let mut from_sq = 0u8;

            for p in 0..6 {
                let piece_attackers = position.bitboards[side_offset + p] & attackers;
                if piece_attackers != 0 {
                    from_sq = piece_attackers.trailing_zeros() as u8;
                    attacker = Piece::from(side_offset + p);
                    found = true;
                    break;
                }
            }

            if !found {
                break;
            }

            // Remove this attacker from occupancy
            occ ^= 1u64 << from_sq;

            // Recompute attackers with updated occupancy (discovers x-ray attacks)
            attackers = position.get_all_attackers_to(target, occ) & occ;

            side = !side;
        }

        // Propagate back with negamax logic
        while d > 1 {
            d -= 1;
            gain[d] = -((-gain[d]).max(gain[d + 1]));
        }

        gain[0]
    }

    /// Get the TT move for the current position (from any bound type)
    fn get_tt_move(&self, position: &Position, tt: &TranspositionTable) -> BitPackedMove {
        if let Some(tt_entry) = tt.get(position.hash) {
            let tt_move = tt_entry.get_move();
            if tt_move != BitPackedMove::default() {
                return tt_move;
            }
        }
        BitPackedMove::default()
    }

    /// Score moves using various heuristics. Returns scored move list for pick-best.
    fn score_moves(
        &self,
        moves: Vec<BitPackedMove>,
        position: &Position,
        last_move: Option<BitPackedMove>,
        tt: &TranspositionTable,
    ) -> Vec<(BitPackedMove, i32)> {
        let tt_move = self.get_tt_move(position, tt);

        let mut scored: Vec<(BitPackedMove, i32)> = Vec::with_capacity(moves.len());
        for m in moves {
            let score = self.get_move_score(m, tt_move, last_move, position);
            scored.push((m, score));
        }
        scored
    }

    /// Returns a priority score for a move based on various heuristics.
    fn get_move_score(
        &self,
        m: BitPackedMove,
        tt_move: BitPackedMove,
        last_move: Option<BitPackedMove>,
        position: &Position,
    ) -> i32 {
        // TT move is always searched first
        if m == tt_move && tt_move != BitPackedMove::default() {
            return 10_000_000;
        }

        // Captures: good captures high, bad captures very low
        if m.is_capture() {
            let see_score = self.see(position, m);
            if see_score >= 0 {
                // Good capture: MVV-LVA with large offset
                return 1_000_000 + _MVV_LVA[m.get_piece() as usize][m.get_capture() as usize];
            } else {
                // Bad capture: below quiet moves
                return -1_000_000 + _MVV_LVA[m.get_piece() as usize][m.get_capture() as usize];
            }
        }

        // Queen promotions
        if m.is_promotion() {
            return 900_000;
        }

        // Killer moves
        if self.result.ply < MAX_PLY as u32 {
            if self.killer_moves[0][self.result.ply as usize] == m {
                return 800_001;
            }
            if self.killer_moves[1][self.result.ply as usize] == m {
                return 800_000;
            }
        }

        // Counter move
        if let Some(last) = last_move {
            let counter = self.counter_move_table[last.get_from() as usize][last.get_to() as usize];
            if counter == m && counter != BitPackedMove::default() {
                return 700_000;
            }
        }

        // History heuristic (by piece and destination square)
        self.history_moves[m.get_piece() as usize][m.get_to() as usize]
    }

    pub fn print_info(
        &self,
        start_time: u128,
        score: i32,
        pv_line: &[BitPackedMove],
        tt: &TranspositionTable,
    ) {
        let stop_time: u128 = Evaluator::_get_time_ms();
        let nps: i32 =
            (self.result.nodes as f64 / ((stop_time - start_time) as f64 / 1000.0)) as i32;

        if !pv_line.is_empty() {
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

            if !self.silent {
                print!(
                    "info score {} {} depth {} nodes {} nps {} time {} hashfull {}",
                    if is_mate { "mate" } else { "cp" },
                    if is_mate { mate_in } else { score },
                    self.result.depth,
                    self.result.nodes,
                    nps,
                    stop_time - start_time,
                    tt.get_hashfull()
                );

                let mut pv_str: String = String::new();

                for m in pv_line {
                    pv_str.push_str(" ");
                    pv_str.push_str(m.to_string().as_str());
                }

                println!(" pv{}", pv_str);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        }
    }

    fn check_time(&mut self) -> bool {
        if self.is_stopped() {
            return false;
        }

        // Detect ponderhit transition: ponder_flag goes from true to false
        if let Some(ref flag) = self.ponder_flag {
            if !flag.load(Ordering::SeqCst) {
                // ponderhit received: switch from ponder to timed search
                self.options.ponder = false;
                self.started_at = Evaluator::_get_time_ms();
                self.set_move_time(self.search_turn);
                self.ponder_flag = None;
            }
        }

        if self.maximum_time == u32::MAX {
            return true;
        }

        let elapsed = (Evaluator::_get_time_ms() - self.started_at) as u32;
        elapsed < self.maximum_time
    }

    fn elapsed_ms(&self) -> u32 {
        (Evaluator::_get_time_ms() - self.started_at) as u32
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

    pub fn evaluate(&mut self, position: &mut Position) -> i32 {
        let material_score = position.material[position.turn as usize]
            - position.material[(!position.turn) as usize];

        return material_score
            + self.evaluate_pawn_structure(position)
            + self.evaluate_open_files(position)
            + self.evaluate_king_safety(position)
            + self.evaluate_bishop_pair(position);
    }

    /// Bishop pair bonus: having two bishops is worth extra material
    fn evaluate_bishop_pair(&self, position: &Position) -> i32 {
        let white_bishops =
            (position.bitboards[Piece::WhiteBishop as usize]).count_ones() as i32;
        let black_bishops =
            (position.bitboards[Piece::BlackBishop as usize]).count_ones() as i32;

        let white_bonus = if white_bishops >= 2 { BISHOP_PAIR_BONUS } else { 0 };
        let black_bonus = if black_bishops >= 2 { BISHOP_PAIR_BONUS } else { 0 };

        if position.turn == Color::White {
            white_bonus - black_bonus
        } else {
            black_bonus - white_bonus
        }
    }

    pub fn evaluate_pawn_structure(&mut self, position: &mut Position) -> i32 {
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

        return if position.turn == Color::White {
            white_score - black_score
        } else {
            black_score - white_score
        };
    }

    pub fn evaluate_open_files(&mut self, position: &mut Position) -> i32 {
        let mut white_score = 0;
        let mut black_score = 0;

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

        return if position.turn == Color::White {
            white_score - black_score
        } else {
            black_score - white_score
        };
    }

    pub fn evaluate_king_safety(&mut self, position: &mut Position) -> i32 {
        let white_kings = position.bitboards[Piece::WhiteKing as usize];
        let black_kings = position.bitboards[Piece::BlackKing as usize];
        let white_pawns = position.bitboards[Piece::WhitePawn as usize];
        let black_pawns = position.bitboards[Piece::BlackPawn as usize];

        let white_score = if white_kings != 0 {
            let king_sq = white_kings.trailing_zeros() as usize;
            (position.king_attacks[king_sq] & white_pawns).count_ones() as i32 * 6
        } else {
            0
        };

        let black_score = if black_kings != 0 {
            let king_sq = black_kings.trailing_zeros() as usize;
            (position.king_attacks[king_sq] & black_pawns).count_ones() as i32 * 6
        } else {
            0
        };

        let diff = white_score - black_score;
        let multiplier = 1 - 2 * (position.turn as i32);
        diff * -multiplier
    }
}

/// Runs a Lazy SMP parallel search with the given number of threads.
/// All threads share the transposition table; each gets its own Evaluator and Position clone.
/// Thread 0 is the main thread (prints UCI output, controls timing).
/// Helper threads search at staggered depths for diversity.
pub fn search_parallel(
    position: &Position,
    options: SearchOptions,
    tt: &TranspositionTable,
    stop_flag: &Arc<AtomicBool>,
    ponder_flag: Option<Arc<AtomicBool>>,
    num_threads: usize,
) -> (Option<BitPackedMove>, Option<BitPackedMove>) {
    tt.increment_age();

    if num_threads <= 1 {
        let mut evaluator = Evaluator::new();
        let mut pos = position.clone();
        return evaluator.get_best_move(&mut pos, options, tt, stop_flag, ponder_flag);
    }

    let result: Mutex<(Option<BitPackedMove>, Option<BitPackedMove>)> = Mutex::new((None, None));

    std::thread::scope(|s| {
        for thread_id in 0..num_threads {
            let stop = Arc::clone(stop_flag);
            let ponder = ponder_flag.clone();
            let result_ref = &result;

            s.spawn(move || {
                let mut evaluator = Evaluator::new();
                let mut pos = position.clone();

                if thread_id != 0 {
                    evaluator.set_silent(true);
                    evaluator.is_main_thread = false;
                    // Odd helper threads start one depth deeper for search diversity
                    evaluator.depth_offset = if thread_id % 2 == 1 { 1 } else { 0 };
                }

                let (best, ponder) = evaluator.get_best_move(
                    &mut pos, options, tt, &stop, ponder,
                );

                if thread_id == 0 {
                    *result_ref.lock().unwrap() = (best, ponder);
                }
            });
        }
    });

    result.into_inner().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::chess::constants::STARTING_FEN;
    use crate::chess::square::Square;

    #[test]
    fn short_movetime_returns_legal_move() {
        let mut position = Position::new(Some(STARTING_FEN));
        let mut evaluator = Evaluator::new();
        let tt = TranspositionTable::new(32);
        let stop_flag = Arc::new(AtomicBool::new(false));

        let mut options = SearchOptions::new();
        options.movetime = Some(1); // 1ms - very short time

        tt.increment_age();
        let (best_move, _ponder_move) = evaluator.get_best_move(&mut position, options, &tt, &stop_flag, None);

        assert!(best_move.is_some());
        let m = best_move.unwrap();

        // The move should not be the default a8a8 move
        assert!(
            !(m.get_from() == Square::A8 && m.get_to() == Square::A8),
            "Expected a legal move, got default a8a8"
        );

        // Verify the move is actually legal by trying to make it
        let is_legal = position.make_move(m, false);
        assert!(is_legal, "Expected a legal move, but move was illegal");
    }
}
