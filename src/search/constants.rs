pub const GET_RANK: [u8; 64] = [
    7, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4,
    3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub static _MVV_LVA: [[i32; 12]; 12] = [
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

pub const MAX_PLY: usize = 64;

pub const _REDUCTION_LIMIT: u8 = 3;
pub const _FULL_DEPTH_MOVES: u8 = 3;

pub const DOUBLED_PAWN_PENALTY: i32 = -15;
pub const ISOLATED_PAWN_PENALTY: i32 = -15;
pub const PASSED_PAWN_BONUS: [i32; 8] = [0, 5, 10, 20, 35, 60, 100, 200];

pub const SEMI_OPEN_FILE_SCORE: i32 = 10;
pub const OPEN_FILE_SCORE: i32 = 20;

pub const FUTILITY_MARGIN: i32 = 200;

/// Reverse futility pruning margin per depth (depth * RFP_MARGIN)
pub const RFP_MARGIN: i32 = 80;

/// Razoring margin: if static_eval + RAZOR_MARGIN < alpha at low depth, drop to qsearch
pub const RAZOR_MARGIN: i32 = 300;

/// Delta pruning margin for quiescence search
pub const DELTA_MARGIN: i32 = 200;

/// Late move pruning thresholds by depth (index = depth)
/// At depth d, prune quiet moves after LMP_MOVE_COUNTS[d] moves searched
pub const LMP_MOVE_COUNTS: [usize; 8] = [0, 5, 8, 12, 17, 23, 30, 38];

/// SEE piece values (indexed by Piece enum: P=0,N=1,B=2,R=3,Q=4,K=5,p=6..k=11,Empty=12)
pub const SEE_PIECE_VALUES: [i32; 13] = [
    100, 300, 300, 500, 900, 20000,
    100, 300, 300, 500, 900, 20000,
    0,
];

/// Bishop pair bonus (per side)
pub const BISHOP_PAIR_BONUS: i32 = 30;
